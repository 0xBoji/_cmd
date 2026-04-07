use crate::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Renders the entire TUI. 
/// 
/// Ratatui uses an immediate-mode rendering model. This means the UI is 
/// redrawn entirely every frame. The state (AppState) is managed outside 
/// the render loop, and the rendering function (ui) simply projects 
/// that state into widgets.
pub fn render(f: &mut Frame, app: &AppState) {
    // 1. Create Layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left: Mesh List
            Constraint::Percentage(70), // Right: Events + Log Focus
        ])
        .split(f.size());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Top-Right: Event Stream
            Constraint::Percentage(40), // Bottom-Right: Log Focus
        ])
        .split(chunks[1]);

    // 2. Render Left Panel: Mesh List
    render_mesh_list(f, app, chunks[0]);

    // 3. Render Top-Right Panel: Event Stream
    render_event_stream(f, app, right_chunks[0]);

    // 4. Render Bottom-Right Panel: Log Focus
    render_log_focus(f, app, right_chunks[1]);
}

fn render_mesh_list(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .agents
        .values()
        .enumerate()
        .map(|(i, agent)| {
            let style = if i == app.selected_agent_idx {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match agent.status.as_str() {
                "Idle" => Color::Green,
                "Busy" => Color::Red,
                _ => Color::Gray,
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{:<15}", agent.id), style),
                    Span::styled(format!("{:<10}", agent.role), Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{}", agent.status.as_str()), Style::default().fg(status_color)),
                ]),
                Line::from(vec![
                    Span::styled(if agent.git_locked { " [GIT_LOCK]" } else { "" }, Style::default().fg(Color::Red)),
                ]),
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" [ Mesh List ] ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    f.render_widget(list, area);
}

fn render_event_stream(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .events
        .iter()
        .map(|event| {
            let time = event.timestamp.format("%H:%M:%S").to_string();
            let content = Line::from(vec![
                Span::styled(format!("[{}] ", time), Style::default().fg(Color::Gray)),
                Span::styled(format!("{}: ", event.agent_id), Style::default().fg(Color::Blue)),
                Span::styled(format!("{}", event.kind), Style::default().fg(Color::Magenta)),
                Span::styled(format!(" -> {}", event.payload), Style::default().fg(Color::White)),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" [ Event Stream ] ").borders(Borders::ALL));

    f.render_widget(list, area);
}

fn render_log_focus(f: &mut Frame, app: &AppState, area: Rect) {
    let selected_id = app.get_selected_agent_id();
    
    let content = if let Some(ref id) = selected_id {
        let focused_events: Vec<String> = app
            .events
            .iter()
            .filter(|e| &e.agent_id == id)
            .map(|e| format!("[{}] {}: {}", e.timestamp.format("%H:%M:%S"), e.kind, e.payload))
            .collect();

        if focused_events.is_empty() {
            format!("No recent activity recorded for agent '{}'.", id)
        } else {
            focused_events.join("\n")
        }
    } else {
        "No agent selected.".to_string()
    };

    let title = format!(" [ Log Focus: {} ] ", selected_id.unwrap_or_else(|| "N/A".to_string()));

    let p = Paragraph::new(content)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}
