use std::collections::VecDeque;

#[test]
fn directory_jumps_should_resolve_relative_cd_commands_against_cwd() {
    let entries = vec![core::history::HistoryEntry {
        command: "cd ../project-b".to_string(),
        cwd: Some("/Users/me/work/project-a".to_string()),
        timestamp_unix_ms: Some(1),
    }];

    let jumps = core::history::directory_jump_history_from_entries(&entries);

    assert_eq!(
        jumps.into_iter().collect::<Vec<_>>(),
        vec!["/Users/me/work/project-b".to_string()]
    );
}

#[test]
fn best_directory_jump_should_prefer_frequency_then_recency() {
    let history = VecDeque::from(vec![
        "/tmp/alpha".to_string(),
        "/tmp/visual-alpha".to_string(),
        "/tmp/visual-alpha".to_string(),
        "/tmp/visual-beta".to_string(),
    ]);

    let suggestion = core::history::best_directory_jump_match(&history, "vi");

    assert_eq!(suggestion.as_deref(), Some("/tmp/visual-alpha"));
}
