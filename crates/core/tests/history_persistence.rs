use core::app::TerminalManager;

#[test]
fn load_history_should_keep_recent_unique_commands_in_recency_order() {
    let root = std::env::temp_dir().join(format!(
        "view-history-load-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let file = root.join("history.jsonl");

    std::fs::create_dir_all(&root).expect("history dir");
    std::fs::write(
        &file,
        [
            r#"{"command":"ls"}"#,
            r#"{"command":"cd /tmp"}"#,
            r#"{"command":"ls"}"#,
            r#"{"command":"git status"}"#,
        ]
        .join("\n"),
    )
    .expect("history file");

    let history = core::history::load_history_from_path(&file).expect("load history");

    assert_eq!(
        history.into_iter().collect::<Vec<_>>(),
        vec![
            "cd /tmp".to_string(),
            "ls".to_string(),
            "git status".to_string(),
        ]
    );
}

#[test]
fn append_history_entry_should_create_file_and_keep_last_unique_commands() {
    let root = std::env::temp_dir().join(format!(
        "view-history-append-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let file = root.join("history.jsonl");

    core::history::append_history_entry_to_path(&file, "ls", None).expect("append ls");
    core::history::append_history_entry_to_path(&file, "pwd", None).expect("append pwd");
    core::history::append_history_entry_to_path(&file, "ls", None).expect("append ls again");

    let history = core::history::load_history_from_path(&file).expect("reload history");

    assert_eq!(
        history.into_iter().collect::<Vec<_>>(),
        vec!["pwd".to_string(), "ls".to_string()]
    );
}

#[test]
fn seed_history_should_copy_persisted_commands_into_new_sessions() {
    let mut manager = TerminalManager::new(2);

    manager.seed_history([
        "git status".to_string(),
        "cargo test".to_string(),
        "cargo run -p desktop".to_string(),
    ]);

    let first = manager.sessions[0]
        .history
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let second = manager.sessions[1]
        .history
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    assert_eq!(first, second);
    assert_eq!(
        first,
        vec![
            "git status".to_string(),
            "cargo test".to_string(),
            "cargo run -p desktop".to_string(),
        ]
    );
}
