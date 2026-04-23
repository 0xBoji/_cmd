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
}

pub type TerminalCommandTx = mpsc::UnboundedSender<String>;

pub fn local_shell_command_tx() -> (TerminalCommandTx, mpsc::UnboundedReceiver<String>) {
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

fn shell_zshrc_content() -> &'static str {
    r#"
alias cls='clear'
# OSC 7 directory tracking for terminal emulators
precmd() {
  # Emit OSC 7 escape sequence with path and a newline to ensure reader processes it
  print -Pn "\e]7;file://%m%d\a\n"
}
"#
}

pub async fn start_local_shell(
    session_id: usize,
    cwd: PathBuf,
    event_tx: mpsc::UnboundedSender<TerminalEvent>,
    mut command_rx: mpsc::UnboundedReceiver<String>,
) -> Result<()> {
    let shell_home = PathBuf::from("/tmp/view-shell");
    let _ = tokio::fs::create_dir_all(&shell_home).await;

    // Write .zshrc for alias and OSC 7 directory tracking
    let _ = tokio::fs::write(shell_home.join(".zshrc"), shell_zshrc_content()).await;

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
                if let Some(path) = osc7_path_from_line(&line) {
                    let _ = event_tx.send(TerminalEvent::Cwd {
                        session_id,
                        cwd: path,
                    });
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

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        local_shell_command_tx, osc7_path_from_line, sanitize_terminal_line, shell_zshrc_content,
    };

    #[test]
    fn local_shell_command_channel_should_send_commands() {
        let (tx, mut rx) = local_shell_command_tx();
        tx.send("echo test".to_string()).expect("send");
        assert_eq!(rx.try_recv().expect("recv"), "echo test".to_string());
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
    fn shell_zshrc_should_not_define_cd_dotdot_alias() {
        let zshrc = shell_zshrc_content();
        assert!(!zshrc.contains("alias cd..="));
        assert!(zshrc.contains("alias cls='clear'"));
    }

    #[test]
    fn osc7_path_from_line_should_parse_directory_updates() {
        let line = "\u{1b}]7;file://host/Users/me/project\u{7}";
        assert_eq!(
            osc7_path_from_line(line).as_deref(),
            Some("/Users/me/project")
        );
    }
}
