use crate::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline, Wrap},
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
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Top: Header
            Constraint::Min(0),         // Middle: Body
            Constraint::Length(1),      // Bottom: Footer
        ])
        .split(f.size());

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Left: Agent List + Sparkline
            Constraint::Percentage(75), // Right: Metrics + Logs
        ])
        .split(main_chunks[1]);

    // 2. Render Components
    render_header(f, app, main_chunks[0]);
    render_left_pane(f, app, body_chunks[0]);
    render_right_pane(f, app, body_chunks[1]);
    render_footer(f, main_chunks[2]);
}

fn render_left_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),         // Agent List
            Constraint::Length(7),      // Sparkline area
        ])
        .split(area);

    render_mesh_list(f, app, chunks[0]);
    render_activity_sparkline(f, app, chunks[1]);
}

fn render_right_pane(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),      // Metrics Summary
            Constraint::Min(0),         // Execution Logs
        ])
        .split(area);

    render_metrics_summary(f, app, chunks[0]);
    render_log_focus(f, app, chunks[1]);
}

fn render_header(f: &mut Frame, app: &AppState, area: Rect) {
    let mesh_count = app.agents.len();
    let event_total = app.total_events_received;

    let content = Line::from(vec![
        Span::styled(" VIEW ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::styled(format!("Mesh: {} agents", mesh_count), Style::default().fg(Color::Cyan)),
        Span::raw(" │ "),
        Span::styled(format!("Total Events: {}", event_total), Style::default().fg(Color::Green)),
        Span::raw(" │ "),
        Span::styled("MODE: Real-time (CAMP)", Style::default().fg(Color::Yellow)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    
    let p = Paragraph::new(content)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let help_text = " [j/k] Navigate │ [q] Quit │ [c] Ctrl+C Exit ";
    let p = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(p, area);
}

fn render_activity_sparkline(f: &mut Frame, app: &AppState, area: Rect) {
    let selected_id = app.get_selected_agent_id();
    let data = if let Some(id) = selected_id {
        if let Some(agent) = app.agents.get(&id) {
            let (s1, s2) = agent.activity.as_slices();
            let mut combined = s1.to_vec();
            combined.extend_from_slice(s2);
            combined
        } else {
            vec![0; 50]
        }
    } else {
        vec![0; 50]
    };

    let sparkline = Sparkline::default()
        .block(Block::default().title(" [ Activity Frequency (50s) ] ").borders(Borders::ALL))
        .data(&data)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(sparkline, area);
}

fn render_metrics_summary(f: &mut Frame, app: &AppState, area: Rect) {
    let selected_id = app.get_selected_agent_id();
    let agent = selected_id.and_then(|id| app.agents.get(&id));

    let content = if let Some(agent) = agent {
        let status_style = match agent.status.as_str() {
            "Busy" | "busy" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            "Offline" => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::Green),
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled(" [ Agent Details ] ", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw(" - ID:       "),
                Span::styled(&agent.id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw(" - Role:     "),
                Span::raw(&agent.role),
            ]),
            Line::from(vec![
                Span::raw(" - Status:   "),
                Span::styled(&agent.status, status_style),
            ]),
            Line::from(vec![
                Span::raw(" - Tokens:   "),
                Span::styled(format_tokens(agent.tokens), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw(" - Branch:   "),
                Span::styled(&agent.branch, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::raw(" - Proj:     "),
                Span::raw(&agent.project),
            ]),
        ];

        if agent.status == "busy" || agent.status == "Busy" {
            lines.insert(1, Line::from(vec![
                Span::styled(" ● EXECUTING ", Style::default().fg(Color::Red).add_modifier(Modifier::SLOW_BLINK)),
            ]));
        }

        Text::from(lines)
    } else {
        Text::from("No agent selected to view metrics.")
    };

    let p = Paragraph::new(content)
        .block(Block::default().title(" [ Metrics Summary ] ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}

fn format_tokens(tokens: u64) -> String {
    let s = tokens.to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let mut result = Vec::new();
    for (i, c) in chars.into_iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.into_iter().rev().collect()
}

fn render_mesh_list(f: &mut Frame, app: &AppState, area: Rect) {
    let items: Vec<ListItem> = app
        .agents
        .values()
        .enumerate()
        .map(|(i, agent)| {
            let base_style = if i == app.selected_agent_idx {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if agent.status == "Offline" {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match agent.status.as_str() {
                "Idle" | "idle" => Color::Green,
                "Busy" | "busy" => Color::Red,
                "Offline" => Color::DarkGray,
                _ => Color::Cyan,
            };

            let content = Line::from(vec![
                Span::styled(format!("{:<15}", agent.id), base_style),
                Span::styled(format!("{}", agent.status), Style::default().fg(status_color)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" [ Mesh List ] ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    f.render_widget(list, area);
}

// render_event_stream is removed in Phase 3

fn render_log_focus(f: &mut Frame, app: &AppState, area: Rect) {
    let selected_id = app.get_selected_agent_id();
    
    let content = if let Some(ref id) = selected_id {
        let agent_events = app.get_events_for_agent(id);

        if agent_events.is_empty() {
            Text::from(format!("No recent activity recorded for agent '{}'.", id))
        } else {
            let mut lines = Vec::new();
            for e in agent_events {
                let level_color = match e.level.as_str() {
                    "error" => Color::Red,
                    "warn" => Color::Yellow,
                    "success" => Color::Green,
                    _ => Color::Cyan, // info or other
                };

                let component_tag = format!("[{:<6}] ", e.component.to_uppercase());
                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", e.timestamp.format("%H:%M:%S")), Style::default().fg(Color::DarkGray)),
                    Span::styled(component_tag, Style::default().fg(level_color).add_modifier(Modifier::BOLD)),
                    Span::raw(&e.payload),
                ]));
            }
            Text::from(lines)
        }
    } else {
        Text::from("No agent selected.")
    };

    let title = format!(" [ Log Focus: {} ] ", selected_id.unwrap_or_else(|| "N/A".to_string()));

    let p = Paragraph::new(content)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}
