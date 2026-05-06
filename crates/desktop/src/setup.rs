use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const MANAGED_BLOCK_START: &str = "# >>> _cmd managed shell >>>";
const MANAGED_BLOCK_END: &str = "# <<< _cmd managed shell <<<";

pub enum SetupCommand {
    LaunchUi,
    SetupShell,
    ResetShell,
    PrintShellInit,
}

pub struct ShellSetupStatus {
    pub managed_path: PathBuf,
    pub zshrc_path: PathBuf,
    pub managed_file_exists: bool,
    pub zshrc_patched: bool,
}

pub fn parse_command(args: &[String]) -> SetupCommand {
    match args.get(1).map(String::as_str) {
        Some("setup-shell") => SetupCommand::SetupShell,
        Some("reset-shell") => SetupCommand::ResetShell,
        Some("print-shell-init") => SetupCommand::PrintShellInit,
        _ => SetupCommand::LaunchUi,
    }
}

pub fn run_setup_shell() -> Result<()> {
    let managed_path = managed_zsh_path().context("missing HOME for managed zsh path")?;
    let zshrc_path = user_zshrc_path().context("missing HOME for ~/.zshrc path")?;

    if let Some(parent) = managed_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &managed_path,
        core::terminal::managed_zsh_shell_integration(),
    )
    .with_context(|| format!("failed to write {}", managed_path.display()))?;

    let original = fs::read_to_string(&zshrc_path).unwrap_or_default();
    let patched = apply_managed_zshrc_block(&original, &managed_path.display().to_string());
    fs::write(&zshrc_path, patched)
        .with_context(|| format!("failed to write {}", zshrc_path.display()))?;

    println!("Managed shell config written to {}", managed_path.display());
    println!("Patched {}", zshrc_path.display());
    Ok(())
}

pub fn run_reset_shell() -> Result<()> {
    let managed_path = managed_zsh_path().context("missing HOME for managed zsh path")?;
    let zshrc_path = user_zshrc_path().context("missing HOME for ~/.zshrc path")?;

    if let Ok(original) = fs::read_to_string(&zshrc_path) {
        let reset = remove_managed_zshrc_block(&original);
        fs::write(&zshrc_path, reset)
            .with_context(|| format!("failed to write {}", zshrc_path.display()))?;
    }

    if managed_path.exists() {
        let _ = fs::remove_file(&managed_path);
    }

    println!("Removed managed shell block from {}", zshrc_path.display());
    println!("Removed {}", managed_path.display());
    Ok(())
}

pub fn print_shell_init() -> Result<()> {
    println!("{}", preview_shell_init()?);
    Ok(())
}

pub fn inspect_shell_setup() -> Result<ShellSetupStatus> {
    let managed_path = managed_zsh_path().context("missing HOME for managed zsh path")?;
    let zshrc_path = user_zshrc_path().context("missing HOME for ~/.zshrc path")?;
    let zshrc_content = fs::read_to_string(&zshrc_path).unwrap_or_default();

    Ok(ShellSetupStatus {
        managed_file_exists: managed_path.exists(),
        zshrc_patched: zshrc_has_managed_block(&zshrc_content),
        managed_path,
        zshrc_path,
    })
}

pub fn preview_shell_init() -> Result<String> {
    let managed_path = managed_zsh_path().context("missing HOME for managed zsh path")?;
    Ok(managed_block(&managed_path.display().to_string()))
}

fn managed_zsh_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("VIEW_CONFIG_HOME") {
        return Some(PathBuf::from(root).join("zsh/_cmd.zsh"));
    }

    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/_cmd/zsh/_cmd.zsh"))
}

fn user_zshrc_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".zshrc"))
}

fn managed_block(managed_path: &str) -> String {
    format!(
        "{MANAGED_BLOCK_START}\nsource '{}'\n{MANAGED_BLOCK_END}\n",
        managed_path.replace('\'', "'\\''")
    )
}

pub fn zshrc_has_managed_block(content: &str) -> bool {
    content.contains(MANAGED_BLOCK_START) && content.contains(MANAGED_BLOCK_END)
}

pub fn apply_managed_zshrc_block(original: &str, managed_path: &str) -> String {
    let cleaned = remove_managed_zshrc_block(original);
    let mut output = cleaned.trim_end().to_string();
    if !output.is_empty() {
        output.push('\n');
        output.push('\n');
    }
    output.push_str(&managed_block(managed_path));
    output
}

pub fn remove_managed_zshrc_block(original: &str) -> String {
    let mut output = String::new();
    let mut skipping = false;

    for line in original.lines() {
        if line.trim() == MANAGED_BLOCK_START {
            skipping = true;
            continue;
        }
        if line.trim() == MANAGED_BLOCK_END {
            skipping = false;
            continue;
        }
        if !skipping {
            output.push_str(line);
            output.push('\n');
        }
    }

    while output.contains("\n\n\n") {
        output = output.replace("\n\n\n", "\n\n");
    }

    output.trim_end_matches('\n').to_string() + "\n"
}

#[cfg(test)]
mod tests {
    #[test]
    fn patch_zshrc_should_insert_managed_block_once() {
        let original = "export EDITOR=vim\n";
        let managed =
            super::apply_managed_zshrc_block(original, "/Users/me/.config/_cmd/zsh/_cmd.zsh");

        assert!(managed.contains("export EDITOR=vim"));
        assert!(managed.contains("source '/Users/me/.config/_cmd/zsh/_cmd.zsh'"));
        assert_eq!(managed.matches(">>> _cmd managed shell >>>").count(), 1);

        let reapplied =
            super::apply_managed_zshrc_block(&managed, "/Users/me/.config/_cmd/zsh/_cmd.zsh");
        assert_eq!(managed, reapplied);
    }

    #[test]
    fn reset_zshrc_should_remove_managed_block_and_keep_user_content() {
        let with_block = "export EDITOR=vim\n# >>> _cmd managed shell >>>\nsource '/Users/me/.config/_cmd/zsh/_cmd.zsh'\n# <<< _cmd managed shell <<<\nalias gs='git status'\n";

        let reset = super::remove_managed_zshrc_block(with_block);

        assert!(reset.contains("export EDITOR=vim"));
        assert!(reset.contains("alias gs='git status'"));
        assert!(!reset.contains("_cmd managed shell"));
        assert!(!reset.contains("source '/Users/me/.config/_cmd/zsh/_cmd.zsh'"));
    }

    #[test]
    fn zshrc_has_managed_block_should_detect_complete_marker_pair() {
        let with_block = "# >>> _cmd managed shell >>>\nsource '~/.config/_cmd/zsh/_cmd.zsh'\n# <<< _cmd managed shell <<<\n";
        let without_block = "export EDITOR=vim\n";

        assert!(super::zshrc_has_managed_block(with_block));
        assert!(!super::zshrc_has_managed_block(without_block));
    }
}
