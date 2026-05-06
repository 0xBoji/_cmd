use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Line { session_id: usize, line: String },
    Status { session_id: usize, status: String },
    Cwd { session_id: usize, cwd: String },
    Timing { session_id: usize, seconds: f64 },
    LastCommand { session_id: usize, command: String },
    ExitCode { session_id: usize, exit_code: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalCommand {
    Input(String),
    Resize(TerminalSize),
}

pub type TerminalCommandTx = mpsc::UnboundedSender<TerminalCommand>;

pub fn local_shell_command_tx() -> (TerminalCommandTx, mpsc::UnboundedReceiver<TerminalCommand>) {
    mpsc::unbounded_channel()
}

fn osc7_path_from_line(line: &str) -> Option<String> {
    let pos = line.find("\u{1b}]7;file://")?;
    let rest = &line[pos + 13..];
    let end_pos = rest.find('\u{7}')?;
    let url_part = &rest[..end_pos];
    let path = if let Some(slash_pos) = url_part.find('/') {
        if url_part.starts_with('/') {
            url_part
        } else {
            &url_part[slash_pos..]
        }
    } else {
        url_part
    };
    Some(path.to_string())
}

fn osc1337_user_var_from_line(line: &str, key: &str) -> Option<String> {
    let marker = format!("\u{1b}]1337;SetUserVar={key}=");
    let pos = line.find(&marker)?;
    let rest = &line[pos + marker.len()..];
    let end_pos = rest.find('\u{7}')?;
    decode_osc1337_value(&rest[..end_pos])
}

fn decode_osc1337_value(value: &str) -> Option<String> {
    let Some(encoded) = value.strip_prefix("b64:") else {
        return Some(value.to_string());
    };
    let bytes = decode_base64(encoded)?;
    String::from_utf8(bytes).ok().or_else(|| Some(value.to_string()))
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    fn decode_char(ch: u8) -> Option<u8> {
        match ch {
            b'A'..=b'Z' => Some(ch - b'A'),
            b'a'..=b'z' => Some(ch - b'a' + 26),
            b'0'..=b'9' => Some(ch - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }

    let mut output = Vec::with_capacity((bytes.len() / 4) * 3);
    for chunk in bytes.chunks_exact(4) {
        let mut values = [0u8; 4];
        let mut padding = 0usize;

        for (index, byte) in chunk.iter().copied().enumerate() {
            if byte == b'=' {
                values[index] = 0;
                padding += 1;
                continue;
            }
            values[index] = decode_char(byte)?;
        }

        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding == 0 {
            output.push((values[2] << 6) | values[3]);
        }
    }

    Some(output)
}

pub fn managed_zsh_shell_integration() -> &'static str {
    r#"
alias cls='clear'
HISTFILE="$HOME/.zsh_history"
HISTSIZE=5000
SAVEHIST=5000
setopt SHARE_HISTORY
setopt APPEND_HISTORY
setopt INC_APPEND_HISTORY
setopt EXTENDED_HISTORY
setopt HIST_IGNORE_ALL_DUPS
setopt HIST_IGNORE_SPACE

_view_emit_user_var() {
  local encoded
  encoded="$(printf '%s' "$2" | base64 | tr -d '\r\n')"
  printf '\e]1337;SetUserVar=%s=b64:%s\a\n' "$1" "$encoded"
}

_view_apply_winsize() {
  stty cols "$1" rows "$2" 2>/dev/null || return 0
  export COLUMNS="$1"
  export LINES="$2"
  kill -WINCH $$ >/dev/null 2>&1 || true
}

preexec() {
  _view_emit_user_var "view_prompt_state" "command_start"
  _view_emit_user_var "view_last_command" "$1"
}

# OSC 7 directory tracking for terminal emulators
precmd() {
  local exit_code=$?
  if [[ -n "${VIEW_SHELL_INTEGRATION:-}" ]]; then
    _view_emit_user_var "view_prompt_state" "prompt_ready"
    _view_emit_user_var "view_last_exit_code" "$exit_code"
  fi
  # Emit OSC 7 escape sequence with path and a newline to ensure reader processes it
  if [[ -n "${VIEW_SHELL_INTEGRATION:-}" ]]; then
    print -Pn "\e]7;file://%m%d\a\n"
  fi
}
"#
}

fn shell_resize_command(size: TerminalSize) -> String {
    format!(" _view_apply_winsize {} {}", size.cols, size.rows)
}

fn shell_zshrc_content(managed_config_path: &str) -> String {
    format!(
        r#"
if [ -f '{managed_config_path}' ]; then
  source '{managed_config_path}'
fi
"#
    )
}

pub async fn start_local_shell(
    session_id: usize,
    cwd: PathBuf,
    event_tx: mpsc::UnboundedSender<TerminalEvent>,
    mut command_rx: mpsc::UnboundedReceiver<TerminalCommand>,
) -> Result<()> {
    let shell_home = PathBuf::from("/tmp/_cmd-shell");
    let _ = tokio::fs::create_dir_all(&shell_home).await;
    let managed_config_dir = shell_home.join(".config/_cmd/zsh");
    let managed_config_path = managed_config_dir.join("_cmd.zsh");
    let _ = tokio::fs::create_dir_all(&managed_config_dir).await;

    let _ = tokio::fs::write(&managed_config_path, managed_zsh_shell_integration()).await;
    let _ = tokio::fs::write(
        shell_home.join(".zshrc"),
        shell_zshrc_content(&managed_config_path.display().to_string()),
    )
    .await;

    let mut child = Command::new("/usr/bin/script")
        .arg("-q")
        .arg("/dev/null")
        .arg("/bin/zsh")
        .arg("-di")
        .current_dir(&cwd)
        .env("HOME", &shell_home)
        .env("ZDOTDIR", &shell_home)
        .env("TERM", "xterm-256color")
        .env("PS1", "$ ")
        .env("VIEW_SHELL_INTEGRATION", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture shell stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture shell stdout"))?;

    let _ = event_tx.send(TerminalEvent::Status {
        session_id,
        status: "ready".to_string(),
    });
    let _ = event_tx.send(TerminalEvent::Cwd {
        session_id,
        cwd: cwd.display().to_string(),
    });

    let (reader_task, started_at) = {
        let event_tx = event_tx.clone();
        let started_at = Arc::new(Mutex::new(None::<Instant>));
        let reader_started_at = started_at.clone();
        let handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Some(prompt_state) =
                    osc1337_user_var_from_line(&line, "view_prompt_state")
                {
                    let status = match prompt_state.as_str() {
                        "command_start" => Some("running"),
                        "prompt_ready" => Some("ready"),
                        _ => None,
                    };
                    if let Some(status) = status {
                        let _ = event_tx.send(TerminalEvent::Status {
                            session_id,
                            status: status.to_string(),
                        });
                    }
                    if prompt_state == "prompt_ready" {
                        if let Some(started_at) = reader_started_at
                            .lock()
                            .ok()
                            .and_then(|mut slot| slot.take())
                        {
                            let seconds = started_at.elapsed().as_secs_f64();
                            let _ = event_tx.send(TerminalEvent::Timing {
                                session_id,
                                seconds,
                            });
                        }
                    }
                }
                if let Some(command) = osc1337_user_var_from_line(&line, "view_last_command") {
                    let _ = event_tx.send(TerminalEvent::LastCommand {
                        session_id,
                        command,
                    });
                }
                if let Some(exit_code) = osc1337_user_var_from_line(&line, "view_last_exit_code")
                    .and_then(|value| value.parse::<i32>().ok())
                {
                    let _ = event_tx.send(TerminalEvent::ExitCode {
                        session_id,
                        exit_code,
                    });
                }
                if let Some(path) = osc7_path_from_line(&line) {
                    let _ = event_tx.send(TerminalEvent::Cwd {
                        session_id,
                        cwd: path,
                    });
                }

                let cleaned = sanitize_terminal_line(&line);
                if !cleaned.is_empty() {
                    let _ = event_tx.send(TerminalEvent::Line {
                        session_id,
                        line: cleaned,
                    });
                }
            }
        });

        (handle, started_at)
    };

    while let Some(command) = command_rx.recv().await {
        match command {
            TerminalCommand::Input(command) => {
                if let Ok(mut slot) = started_at.lock() {
                    *slot = Some(Instant::now());
                }
                stdin.write_all(command.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;
                let _ = event_tx.send(TerminalEvent::Status {
                    session_id,
                    status: "running".to_string(),
                });
            }
            TerminalCommand::Resize(size) => {
                let command = shell_resize_command(size);
                stdin.write_all(command.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;
            }
        }
    }

    let _ = reader_task.await;
    let _ = event_tx.send(TerminalEvent::Status {
        session_id,
        status: "closed".to_string(),
    });
    Ok(())
}

fn sanitize_terminal_line(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let chars = line.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        match chars[index] {
            '\u{8}' => {
                output.pop();
                index += 1;
            }
            '\u{1b}' => {
                index += 1;
                if index < chars.len() {
                    match chars[index] {
                        '[' => {
                            // CSI sequence: ESC [ ... [a-zA-Z]
                            index += 1;
                            while index < chars.len() {
                                let ch = chars[index];
                                index += 1;
                                if ('@'..='~').contains(&ch) {
                                    break;
                                }
                            }
                        }
                        ']' => {
                            // OSC sequence: ESC ] ... BEL (or ESC \)
                            index += 1;
                            while index < chars.len() {
                                let ch = chars[index];
                                index += 1;
                                if ch == '\u{7}' {
                                    // BEL
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            '\r' => {
                output.clear();
                index += 1;
            }
            '\t' => {
                // Emulate 8-space tab stops
                let current_width = output.chars().count();
                let spaces_to_add = 8 - (current_width % 8);
                for _ in 0..spaces_to_add {
                    output.push(' ');
                }
                index += 1;
            }
            ch if ch.is_control() => {
                index += 1;
            }
            ch => {
                output.push(ch);
                index += 1;
            }
        }
    }

    let cleaned = output.trim_end().to_string();
    if cleaned.contains("_view_apply_winsize ") {
        return String::new();
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::{
        local_shell_command_tx, managed_zsh_shell_integration, osc1337_user_var_from_line,
        osc7_path_from_line, sanitize_terminal_line, shell_zshrc_content, TerminalCommand,
    };

    #[test]
    fn local_shell_command_channel_should_send_commands() {
        let (tx, mut rx) = local_shell_command_tx();
        tx.send(TerminalCommand::Input("echo test".to_string()))
            .expect("send");
        assert_eq!(
            rx.try_recv().expect("recv"),
            TerminalCommand::Input("echo test".to_string())
        );
    }

    #[test]
    fn sanitize_terminal_line_should_strip_ansi_and_backspaces() {
        let line = "\u{1b}[32mVIEW\u{1b}[0m sh\u{8}hell ready\r$ echo hi";
        assert_eq!(sanitize_terminal_line(line), "$ echo hi");
    }

    #[test]
    fn sanitize_terminal_line_should_drop_empty_escape_only_lines() {
        let line = "\u{1b}[?2004h\u{1b}[?2004l";
        assert_eq!(sanitize_terminal_line(line), "");
    }

    #[test]
    fn sanitize_terminal_line_should_hide_internal_resize_commands() {
        let line = "$  _view_apply_winsize 132 40";
        assert_eq!(sanitize_terminal_line(line), "");
    }

    #[test]
    fn shell_zshrc_should_not_define_cd_dotdot_alias() {
        let zshrc = managed_zsh_shell_integration();
        assert!(!zshrc.contains("alias cd..="));
        assert!(zshrc.contains("alias cls='clear'"));
        assert!(zshrc.contains("setopt SHARE_HISTORY"));
        assert!(zshrc.contains("setopt INC_APPEND_HISTORY"));
        assert!(zshrc.contains("setopt HIST_IGNORE_SPACE"));
        assert!(zshrc.contains("preexec()"));
        assert!(zshrc.contains("view_last_exit_code"));
        assert!(zshrc.contains("view_prompt_state"));
        assert!(zshrc.contains("b64:"));
        assert!(zshrc.contains("_view_apply_winsize()"));
        assert!(zshrc.contains("stty cols \"$1\" rows \"$2\""));
        assert!(!zshrc.contains("builtin stty"));
    }

    #[test]
    fn shell_zshrc_loader_should_source_managed_config() {
        let zshrc = shell_zshrc_content("/tmp/_cmd-shell/.config/_cmd/zsh/_cmd.zsh");
        assert!(zshrc.contains("source '/tmp/_cmd-shell/.config/_cmd/zsh/_cmd.zsh'"));
    }

    #[test]
    fn osc7_path_from_line_should_parse_directory_updates() {
        let line = "\u{1b}]7;file://host/Users/me/project\u{7}";
        assert_eq!(
            osc7_path_from_line(line).as_deref(),
            Some("/Users/me/project")
        );
    }

    #[test]
    fn osc1337_user_var_from_line_should_parse_shell_metadata() {
        let line = "\u{1b}]1337;SetUserVar=view_last_command=git status\u{7}";
        assert_eq!(
            osc1337_user_var_from_line(line, "view_last_command").as_deref(),
            Some("git status")
        );
    }

    #[test]
    fn osc1337_user_var_from_line_should_decode_base64_payloads() {
        let line = "\u{1b}]1337;SetUserVar=view_last_command=b64:Z2l0IHN0YXR1cyAtLXNob3J0\u{7}";
        assert_eq!(
            osc1337_user_var_from_line(line, "view_last_command").as_deref(),
            Some("git status --short")
        );
    }
}
