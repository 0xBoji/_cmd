use core::{
    app::{AppState, PaneRect},
    engine::{Action, CoreEngine},
    terminal::TerminalSize,
};
use eframe::egui::text_selection::LabelSelectionState;
use eframe::egui::{
    self, Align, Color32, Event, Frame, Key, Layout, PointerButton, RichText, ScrollArea, Stroke,
    UiBuilder, ViewportCommand,
};
use egui_extras;
use image::{ImageBuffer, Rgba};
use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;
use tokio::sync::mpsc;

use crate::{
    setup, shell, shortcuts,
    transcript::{
        command_block_has_error, is_command_context_line, is_context_block_start,
        is_error_output_line, is_legacy_context_block_start, should_extend_error_block_to_bottom,
        should_render_block_separator, wrap_columns_for_width,
    },
};

use crate::theme::*;

use crate::utils::layout::*;
use crate::utils::search::*;
use crate::panels::settings::*;


pub struct CmdDesktopApp {
    state: Arc<RwLock<AppState>>,
    shell_input: String,
    history_offset: usize,
    directory_picker_open: bool,
    directory_picker_query: String,
    branch_picker_open: bool,
    branch_picker_query: String,
    action_tx: mpsc::UnboundedSender<Action>,
    frame_count: u64,
    screenshot_requested: bool,
    screenshot_target: Option<PathBuf>,
    copied_badge_until: Option<Instant>,
    selection_drag_origin: Option<egui::Pos2>,
    selected_transcript_blocks: BTreeSet<usize>,
    terminal_find_open: bool,
    terminal_find_query: String,
    terminal_find_case_sensitive: bool,
    terminal_find_selected_only: bool,
    terminal_find_active_result: Option<usize>,
    shell_setup_open: bool,
    shell_setup_show_preview: bool,
    shell_setup_status_message: Option<(String, bool)>,
    settings_section: SettingsSection,
    last_observed_terminal_size: Option<TerminalSize>,
}

impl CmdDesktopApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        debug_log("desktop app: new()".to_string());
        configure_theme(&cc.egui_ctx);
        // Install SVG/image loaders so egui can render .svg assets (Warp-style icons)
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let state = Arc::new(RwLock::new(AppState::new()));
        let action_tx = spawn_core_runtime(state.clone());

        Self {
            state,
            shell_input: String::new(),
            history_offset: 0,
            directory_picker_open: false,
            directory_picker_query: String::new(),
            branch_picker_open: false,
            branch_picker_query: String::new(),
            action_tx,
            frame_count: 0,
            screenshot_requested: false,
            screenshot_target: screenshot_target(),
            copied_badge_until: None,
            selection_drag_origin: None,
            selected_transcript_blocks: BTreeSet::new(),
            terminal_find_open: false,
            terminal_find_query: String::new(),
            terminal_find_case_sensitive: false,
            terminal_find_selected_only: false,
            terminal_find_active_result: None,
            shell_setup_open: false,
            shell_setup_show_preview: false,
            shell_setup_status_message: None,
            settings_section: SettingsSection::Shell,
            last_observed_terminal_size: None,
        }
    }
}

// suggestion helper — delegated to shell module
#[inline]
fn terminal_suggestion_suffix(input: &str, suggestion: Option<&str>) -> Option<String> {
    shell::terminal_suggestion_suffix(input, suggestion)
}

// shell_quote_path — delegated to shell module
#[inline]
fn shell_quote_path(path: &str) -> String {
    shell::shell_quote_path(path)
}

fn directory_picker_options(cwd: &str, query: &str) -> Vec<shell::DirectoryOption> {
    shell::directory_picker_options(cwd, query)
}

fn branch_picker_options(cwd: &str, query: &str) -> Vec<shell::BranchOption> {
    shell::branch_picker_options(cwd, query)
}

fn submit_shell_command(
    state: &mut AppState,
    action_tx: mpsc::UnboundedSender<Action>,
    history_offset: &mut usize,
    command: String,
) -> bool {
    shell::submit_shell_command(state, action_tx, history_offset, command)
}

fn history_entry_for_offset(
    history: &std::collections::VecDeque<String>,
    history_offset: usize,
) -> Option<String> {
    shell::history_entry_for_offset(history, history_offset)
}



use crate::components::buttons::*;
use crate::components::badges::*;
use crate::components::dividers::*;
use crate::components::tooltips::*;






fn picker_height(option_count: usize, row_height: f32, max_visible_rows: usize) -> f32 {
    let visible_rows = option_count.max(1).min(max_visible_rows) as f32;
    52.0 + (visible_rows * row_height)
}

fn fill_gap(ui: &mut egui::Ui, height: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    if color != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, color);
    }
}

fn update_selected_blocks(selected: &mut BTreeSet<usize>, clicked: usize, extend_selection: bool) {
    if extend_selection {
        if !selected.insert(clicked) {
            selected.remove(&clicked);
        }
        return;
    }

    if selected.len() == 1 && selected.contains(&clicked) {
        selected.clear();
    } else {
        selected.clear();
        selected.insert(clicked);
    }
}

fn transcript_block_fill_color(has_error: bool, is_selected: bool) -> Color32 {
    if is_selected {
        SELECTED_PANEL_BG
    } else if has_error {
        ERROR_PANEL_BG
    } else {
        Color32::TRANSPARENT
    }
}



fn shell_input_should_submit(
    directory_picker_open: bool,
    branch_picker_open: bool,
    input_has_focus: bool,
    input_lost_focus: bool,
    enter_pressed: bool,
) -> bool {
    !directory_picker_open
        && !branch_picker_open
        && enter_pressed
        && (input_has_focus || input_lost_focus)
}



fn apply_match_highlights(
    job: &mut egui::text::LayoutJob,
    highlight_ranges: &[std::ops::Range<usize>],
    active_highlight: Option<std::ops::Range<usize>>,
) {
    if highlight_ranges.is_empty() {
        return;
    }

    let mut split_points = vec![0usize, job.text.len()];
    for range in highlight_ranges {
        split_points.push(range.start);
        split_points.push(range.end);
    }
    if let Some(range) = &active_highlight {
        split_points.push(range.start);
        split_points.push(range.end);
    }
    split_points.sort_unstable();
    split_points.dedup();

    let mut sections = Vec::new();
    for window in split_points.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }
        let Some(base_section) = job
            .sections
            .iter()
            .find(|section| section.byte_range.start <= start && section.byte_range.end >= end)
        else {
            continue;
        };

        let mut format = base_section.format.clone();
        if active_highlight
            .as_ref()
            .is_some_and(|range| range.start <= start && range.end >= end)
        {
            format.background = SEARCH_ACTIVE_HIGHLIGHT_BG;
            format.color = SEARCH_HIGHLIGHT_TEXT;
        } else if highlight_ranges
            .iter()
            .any(|range| range.start <= start && range.end >= end)
        {
            format.background = SEARCH_HIGHLIGHT_BG;
            format.color = SEARCH_HIGHLIGHT_TEXT;
        }

        sections.push(egui::text::LayoutSection {
            leading_space: base_section.leading_space,
            byte_range: start..end,
            format,
        });
    }
    job.sections = sections;
}

fn mono_text_format(color: Color32, strong: bool) -> egui::TextFormat {
    egui::TextFormat {
        font_id: if strong {
            egui::FontId::new(13.0, egui::FontFamily::Monospace)
        } else {
            egui::FontId::monospace(13.0)
        },
        color,
        ..Default::default()
    }
}

fn append_token(job: &mut egui::text::LayoutJob, text: &str, color: Color32, strong: bool) {
    job.append(text, 0.0, mono_text_format(color, strong));
}

fn append_preserving_whitespace(
    job: &mut egui::text::LayoutJob,
    line: &str,
    color_fn: impl Fn(&str) -> Color32,
    strong_fn: impl Fn(&str) -> bool,
) {
    let mut current = String::new();
    let mut in_whitespace = None;
    for ch in line.chars() {
        let is_ws = ch.is_whitespace();
        if in_whitespace == Some(is_ws) || in_whitespace.is_none() {
            current.push(ch);
            in_whitespace = Some(is_ws);
            continue;
        }

        let color = if in_whitespace == Some(true) {
            FG_PRIMARY
        } else {
            color_fn(&current)
        };
        append_token(job, &current, color, strong_fn(&current));
        current.clear();
        current.push(ch);
        in_whitespace = Some(is_ws);
    }

    if !current.is_empty() {
        let color = if in_whitespace == Some(true) {
            FG_PRIMARY
        } else {
            color_fn(&current)
        };
        append_token(job, &current, color, strong_fn(&current));
    }
}

/// Shorten the absolute path at the start of a context line to `../name`.
/// Input:  `/Volumes/0xboji/learn/rust/coding-agent/visual_interception_event_window git:(main)`
/// Output: `../visual_interception_event_window git:(main)`
fn shorten_context_line(line: &str) -> String {
    let mut parts = line.splitn(2, ' ');
    let path = match parts.next() {
        Some(p) if p.starts_with('/') => p,
        _ => return line.to_string(),
    };
    let rest = parts.next();
    let last = path.split('/').filter(|s| !s.is_empty()).last().unwrap_or(path);
    let short = format!("../{last}");
    match rest {
        Some(r) => format!("{short} {r}"),
        None => short,
    }
}

fn context_line_job(line: &str) -> egui::text::LayoutJob {
    let shortened = shorten_context_line(line);
    let mut job = egui::text::LayoutJob::default();
    append_preserving_whitespace(
        &mut job,
        &shortened,
        |token| {
            if token.starts_with("../") {
                PATH_BLUE
            } else if token.starts_with("git:(") {
                BRANCH_GREEN
            } else if token.starts_with('+') {
                POSITIVE_GREEN
            } else if token.starts_with('-') {
                NEGATIVE_RED
            } else {
                FG_MUTED
            }
        },
        |_| false,
    );
    job
}

fn command_line_job(line: &str, has_error: bool) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    append_token(
        &mut job,
        "$ ",
        if has_error {
            ERROR_COMMAND_TEXT
        } else {
            FG_PRIMARY
        },
        true,
    );
    let command = line.strip_prefix("$ ").unwrap_or(line);
    if has_error {
        append_token(&mut job, command, ERROR_COMMAND_TEXT, false);
        return job;
    }
    append_preserving_whitespace(
        &mut job,
        command,
        |token| {
            if token == "git" || token == "cd" || token == "ls" {
                COMMAND_CYAN
            } else if token.starts_with('/') || token.starts_with('.') || token.contains('/') {
                DIRECTORY_PURPLE
            } else if token.starts_with('-') {
                FG_MUTED
            } else if has_error {
                ERROR_TEXT
            } else {
                FG_PRIMARY
            }
        },
        |token| token == "git" || token == "cd" || token == "ls",
    );
    job
}


fn get_terminal_wrapped_lines(line: &str, wrap_cols: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let chars: Vec<char> = line.chars().collect();
    let mut chunks = Vec::new();
    for chunk in chars.chunks(wrap_cols.max(1)) {
        chunks.push(chunk.iter().collect());
    }
    chunks
}

fn output_line_job(line: &str, cwd: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    append_preserving_whitespace(
        &mut job,
        line,
        |token| {
            if is_error_output_line(token) || is_error_output_line(line) {
                ERROR_TEXT
            } else if token == "." || token == ".." || Path::new(cwd).join(token).is_dir() {
                DIRECTORY_PURPLE
            } else {
                FG_PRIMARY
            }
        },
        |_| false,
    );
    job
}

fn terminal_size_for_content_rect(rect: egui::Rect) -> TerminalSize {
    let usable_width = (rect.width() - 36.0).max(320.0);
    let usable_height = (rect.height() - 168.0).max(180.0);
    let cols = (usable_width / 8.4).floor().clamp(40.0, 320.0) as u16;
    let rows = (usable_height / 18.0).floor().clamp(10.0, 160.0) as u16;
    TerminalSize { cols, rows }
}

impl eframe::App for CmdDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(120));
        self.frame_count += 1;
        debug_log(format!("frame={} entering update", self.frame_count));

        if ctx.input(|input| input.pointer.any_pressed()) {
            self.selection_drag_origin = ctx.input(|input| input.pointer.press_origin());
        }

        let dragged_far_enough = ctx.input(|input| {
            let Some(origin) = self.selection_drag_origin else {
                return false;
            };
            let Some(current) = input.pointer.interact_pos() else {
                return false;
            };
            origin.distance(current) > 6.0
        });
        let clicked_word_or_line = ctx.input(|input| {
            input.pointer.button_double_clicked(PointerButton::Primary)
                || input.pointer.button_triple_clicked(PointerButton::Primary)
        });
        let should_copy_selection = ctx.input(|input| input.pointer.any_released())
            && (dragged_far_enough || clicked_word_or_line)
            && ctx.plugin::<LabelSelectionState>().lock().has_selection();
        if should_copy_selection {
            ctx.input_mut(|input| input.events.push(Event::Copy));
            self.copied_badge_until = Some(Instant::now() + Duration::from_secs_f32(1.2));
        }
        if ctx.input(|input| input.pointer.any_released()) {
            self.selection_drag_origin = None;
        }

        if self.shell_setup_open {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.shell_setup_open = false;
                self.shell_setup_status_message = None;
            }
        }

        let mut state = self.state.write();
        let session_count_before = state.terminal_sessions().len();
        shortcuts::handle(ctx, &mut state);
        let session_count_after = state.terminal_sessions().len();

        // If Cmd+T / Cmd+Shift+T added new sessions, spawn shell processes for them.
        if session_count_after > session_count_before {
            // Use the active session's CWD so new panes inherit the current directory.
            let cwd = state
                .selected_terminal()
                .and_then(|s| {
                    if s.cwd.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(&s.cwd))
                    }
                })
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));

            for new_id in session_count_before..session_count_after {
                let _ = self.action_tx.send(Action::SpawnTerminal {
                    cwd: cwd.clone(),
                });
                // Also resize the new session immediately
                if let Some(size) = self.last_observed_terminal_size {
                    let _ = self.action_tx.send(Action::ResizeTerminal {
                        session_id: new_id,
                        size,
                    });
                }
            }
        }

        let terminal_size = terminal_size_for_content_rect(ctx.content_rect());
        if self.last_observed_terminal_size != Some(terminal_size) {
            self.last_observed_terminal_size = Some(terminal_size);
            for session_id in 0..state.terminal_sessions().len() {
                let _ = self.action_tx.send(Action::ResizeTerminal {
                    session_id,
                    size: terminal_size,
                });
            }
        }

        egui::TopBottomPanel::top("desktop_titlebar")
            .show_separator_line(false)
            .exact_height(38.0)
            .frame(
                Frame::new()
                    .fill(TITLEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(12, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let gear = titlebar_icon_button(ui, "⚙", self.shell_setup_open);
                        if gear.clicked() {
                            self.shell_setup_open = !self.shell_setup_open;
                            if !self.shell_setup_open {
                                self.shell_setup_status_message = None;
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG_APP).inner_margin(0.0))
            .show(ctx, |ui| {
                render_focus(
                    ui,
                    &mut state,
                    &mut self.shell_input,
                    &mut self.history_offset,
                    &mut self.selected_transcript_blocks,
                    &mut self.terminal_find_open,
                    &mut self.terminal_find_query,
                    &mut self.terminal_find_case_sensitive,
                    &mut self.terminal_find_selected_only,
                    &mut self.terminal_find_active_result,
                    &mut self.directory_picker_open,
                    &mut self.directory_picker_query,
                    &mut self.branch_picker_open,
                    &mut self.branch_picker_query,
                    &self.action_tx,
                )
            });

        if self.shell_setup_open {
            let screen = ctx.content_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::new("settings_backdrop"),
            ));
            painter.rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 72));

            let settings_window = egui::Window::new("settings_modal")
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .fixed_size(egui::vec2(760.0, 430.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(
                    Frame::new()
                        .fill(SETTINGS_BG)
                        .stroke(Stroke::new(1.0, SETTINGS_BORDER))
                        .corner_radius(14.0)
                        .inner_margin(egui::Margin::symmetric(0, 0)),
                );

            let mut request_close = false;
            let mut divider_y = 0.0;
            let modal_response = settings_window.show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    let header_height = 36.0;
                    let (header_rect, _) = ui.allocate_exact_size(
                        egui::vec2(760.0, header_height),
                        egui::Sense::hover(),
                    );
                    ui.painter().text(
                        egui::pos2(header_rect.min.x + 18.0, header_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Settings",
                        egui::FontId::proportional(22.0),
                        SETTINGS_TEXT_PRIMARY,
                    );

                    ui.add_space(10.0);
                    let (divider_rect, _) = ui.allocate_exact_size(
                        egui::vec2(0.0, 1.0),
                        egui::Sense::hover(),
                    );
                    divider_y = divider_rect.center().y;
                    ui.add_space(10.0);

                    ui.horizontal_top(|ui| {
                            Frame::new()
                                .fill(SETTINGS_SIDEBAR_BG)
                                .inner_margin(egui::Margin::symmetric(12, 16))
                                .show(ui, |ui| {
                                    ui.set_width(180.0);
                                    ui.set_min_height(360.0);
                                    ui.vertical(|ui| {
                                        if settings_sidebar_row(
                                            ui,
                                            "Shell",
                                            "⚙",
                                            self.settings_section == SettingsSection::Shell,
                                        )
                                        .clicked()
                                        {
                                            self.settings_section = SettingsSection::Shell;
                                        }
                                        if settings_sidebar_row(
                                            ui,
                                            "Network",
                                            "◌",
                                            self.settings_section == SettingsSection::Network,
                                        )
                                        .clicked()
                                        {
                                            self.settings_section = SettingsSection::Network;
                                        }
                                        if settings_sidebar_row(
                                            ui,
                                            "Shortcuts",
                                            "⌘",
                                            self.settings_section == SettingsSection::Shortcuts,
                                        )
                                        .clicked()
                                        {
                                            self.settings_section = SettingsSection::Shortcuts;
                                        }
                                    });
                                });

                            ui.add_space(12.0);

                            Frame::new()
                                .fill(SETTINGS_CONTENT_BG)
                                .inner_margin(egui::Margin::symmetric(20, 20))
                                .show(ui, |ui| {
                                    ui.set_width(520.0);
                                    ui.set_min_height(360.0);
                                    ui.vertical(|ui| match self.settings_section {
                                        SettingsSection::Shell => render_shell_settings_panel(
                                            ui,
                                            &mut self.shell_setup_show_preview,
                                            &mut self.shell_setup_status_message,
                                        ),
                                        SettingsSection::Network => {
                                            ui.label(
                                                RichText::new("Network")
                                                    .color(SETTINGS_TEXT_PRIMARY)
                                                    .size(18.0)
                                                    .strong(),
                                            );
                                            ui.add_space(8.0);
                                            ui.label(
                                                RichText::new(
                                                    "Remote access controls can live here next.",
                                                )
                                                .color(SETTINGS_TEXT_SECONDARY)
                                                .size(14.0),
                                            );
                                        }
                                        SettingsSection::Shortcuts => {
                                            ui.label(
                                                RichText::new("Shortcuts")
                                                    .color(SETTINGS_TEXT_PRIMARY)
                                                    .size(18.0)
                                                    .strong(),
                                            );
                                            ui.add_space(8.0);
                                            ui.label(
                                                RichText::new(
                                                    "Keyboard shortcut customization can live here next.",
                                                )
                                                .color(SETTINGS_TEXT_SECONDARY)
                                                .size(14.0),
                                            );
                                        }
                                    });
                                });
                        });
                    });
                });
            if let Some(inner) = modal_response {
                let modal_rect = inner.response.rect;

                ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("settings_modal_divider"),
                ))
                .hline(
                    modal_rect.x_range(),
                    divider_y,
                    Stroke::new(1.0, SETTINGS_BORDER),
                );

                egui::Area::new(egui::Id::new("settings_modal_close"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(modal_rect.max.x - 42.0, modal_rect.min.y + 14.0))
                    .show(ctx, |ui| {
                        let close = titlebar_icon_button(ui, "×", false);
                        if close.clicked() {
                            request_close = true;
                        }
                    });
            }
            if request_close {
                self.shell_setup_open = false;
            }
        }

        let copied_this_frame = ctx.output(|output| {
            output
                .commands
                .iter()
                .any(|command| matches!(command, egui::OutputCommand::CopyText(_)))
        });
        if copied_this_frame {
            self.copied_badge_until = Some(Instant::now() + Duration::from_secs_f32(1.2));
        }
        if self
            .copied_badge_until
            .is_some_and(|until| Instant::now() < until)
        {
            render_copied_badge(ctx);
        }

        // Force disable IME to prevent macOS Vietnamese Telex from intercepting
        // terminal inputs, showing blue composing highlights, and eating spaces.
        ctx.output_mut(|o| o.ime = None);

        drop(state);
        self.maybe_capture_screenshot(ctx);
    }
}

impl CmdDesktopApp {
    fn maybe_capture_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot_target.clone() else {
            return;
        };

        if !self.screenshot_requested && self.frame_count >= 6 {
            debug_log(format!("frame={} requesting screenshot", self.frame_count));
            eprintln!(
                "desktop screenshot: requesting viewport screenshot at frame {}",
                self.frame_count
            );
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default()));
            self.screenshot_requested = true;
            return;
        }

        let events = ctx.input(|input| input.events.clone());
        debug_log(format!(
            "frame={} input_events={}",
            self.frame_count,
            events.len()
        ));
        for event in events {
            if let Event::Screenshot { image, .. } = event {
                debug_log("received screenshot event".to_string());
                eprintln!("desktop screenshot: received screenshot event");
                if let Err(error) = save_color_image(&path, &image) {
                    eprintln!("Failed to save desktop screenshot to {:?}: {}", path, error);
                    debug_log(format!("failed saving screenshot: {error}"));
                } else {
                    eprintln!("Desktop screenshot saved to {:?}", path);
                    debug_log(format!("saved screenshot to {:?}", path));
                }
                ctx.send_viewport_cmd(ViewportCommand::Close);
                break;
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "egui render state is split across app fields"
)]
fn render_focus(
    ui: &mut egui::Ui,
    state: &mut AppState,
    shell_input: &mut String,
    history_offset: &mut usize,
    selected_transcript_blocks: &mut BTreeSet<usize>,
    terminal_find_open: &mut bool,
    terminal_find_query: &mut String,
    terminal_find_case_sensitive: &mut bool,
    terminal_find_selected_only: &mut bool,
    terminal_find_active_result: &mut Option<usize>,
    directory_picker_open: &mut bool,
    directory_picker_query: &mut String,
    branch_picker_open: &mut bool,
    branch_picker_query: &mut String,
    action_tx: &mpsc::UnboundedSender<Action>,
) {
    if state.selected_terminal().is_none() {
        ui.label("No session selected.");
        return;
    }

    let host_rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(host_rect, 0.0, Color32::from_gray(58));
    // do NOT allocate_rect here — let each child pane allocate its own sub-rect
    let pane_layout = state.terminal_pane_layout_equal(
        PaneRect::new(
            host_rect.min.x,
            host_rect.min.y,
            host_rect.width(),
            host_rect.height(),
        ),
        1.0,
    );


    for (session_id, pane_rect) in pane_layout {
        let rect = egui::Rect::from_min_size(
            egui::pos2(pane_rect.x, pane_rect.y),
            egui::vec2(pane_rect.width, pane_rect.height),
        );
        let Some(session) = state.terminal_sessions().get(session_id).cloned() else {
            continue;
        };
        let active = session_id == state.ui.selected_terminal_idx;
        let mut pane_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(("terminal-pane", session_id))
                .max_rect(rect)
                .layout(Layout::top_down(Align::LEFT)),
        );

        // Paint background onto the exact pane rect — no Frame wrapper that could bleed.
        pane_ui.painter().rect_filled(rect, 0.0, BG_APP);

        if active {
            render_focus_terminal(
                &mut pane_ui,
                session_id,
                &session,
                state,
                shell_input,
                history_offset,
                selected_transcript_blocks,
                terminal_find_open,
                terminal_find_query,
                terminal_find_case_sensitive,
                terminal_find_selected_only,
                terminal_find_active_result,
                directory_picker_open,
                directory_picker_query,
                branch_picker_open,
                branch_picker_query,
                action_tx,
            );
        } else {
            render_terminal_preview(&mut pane_ui, session_id, &session, state);
        }
    }
}

fn render_terminal_preview(
    ui: &mut egui::Ui,
    session_id: usize,
    session: &core::app::TerminalState,
    state: &mut AppState,
) {
    let sections = terminal_pane_sections(ui.max_rect());
    let footer_layout = terminal_footer_layout(sections.footer_rect);
    let recent_lines: Vec<&str> = session.lines.iter().map(String::as_str).collect();
    let footer_metrics = terminal_footer_metrics(sections.transcript_rect.height());
    let blocks = terminal_transcript_blocks(&recent_lines);

    ui.spacing_mut().item_spacing.y = 0.0;

    let mut transcript_ui = ui.new_child(
        UiBuilder::new()
            .id_salt(("terminal-preview-transcript", session_id))
            .max_rect(sections.transcript_rect)
            .layout(Layout::top_down(Align::LEFT)),
    );
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(sections.transcript_rect.height())
        .stick_to_bottom(true)
        .show(&mut transcript_ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(0, 0))
                .show(ui, |ui| {
                    let wrap_cols = wrap_columns_for_width((ui.available_width() - 28.0).max(1.0), 8.4);
                    let num_lines = recent_lines
                        .iter()
                        .map(|line| crate::transcript::wrapped_row_count(line, wrap_cols))
                        .sum::<usize>() as f32;
                    let num_prompts = recent_lines.iter().filter(|l| l.starts_with("$ ")).count() as f32;
                    let estimated_height = (num_lines * 14.5) + (num_prompts * 18.0) + 10.0;
                    let remaining_space = ui.available_height() - estimated_height;
                    if remaining_space > 0.0 {
                        ui.add_space(remaining_space);
                    }

                    let mut line_index = 0usize;
                    let mut block_index = 0usize;
                    let mut previous_block_had_error = false;
                    while line_index < recent_lines.len() {
                        if let Some(block) = blocks.get(block_index).copied() {
                            if block.start == line_index {
                                if block_index == 0 {
                                    draw_divider(ui, Color32::from_gray(60));
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, Color32::TRANSPARENT);
                                }
                                let render_separator =
                                    should_render_block_separator(previous_block_had_error, block.has_error);
                                if render_separator && block_index > 0 {
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, Color32::TRANSPARENT);
                                    draw_divider(ui, Color32::from_gray(60));
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, Color32::TRANSPARENT);
                                }

                                let block_width = ui.available_width();
                                let block_padding_y = if block.has_error { 16 } else { 8 };
                                Frame::new()
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .corner_radius(0.0)
                                    .inner_margin(egui::Margin {
                                        left: 14,
                                        right: 14,
                                        top: block_padding_y,
                                        bottom: block_padding_y,
                                    })
                                    .show(ui, |ui| {
                                        ui.set_min_width((block_width - 28.0).max(1.0));
                                        for block_line in &recent_lines[block.start..block.end] {
                                            let wrapped_lines = get_terminal_wrapped_lines(block_line, wrap_cols);
                                            for line_chunk in wrapped_lines {
                                                let job = if is_command_context_line(&line_chunk) {
                                                    context_line_job(&line_chunk)
                                                } else if line_chunk.starts_with("$ ") {
                                                    command_line_job(&line_chunk, block.has_error)
                                                } else if line_chunk.starts_with("~ (") {
                                                    let mut job = egui::text::LayoutJob::default();
                                                    append_token(&mut job, &line_chunk, FG_MUTED, false);
                                                    job
                                                } else {
                                                    output_line_job(&line_chunk, &session.cwd)
                                                };
                                                ui.label(job);
                                            }
                                        }
                                    });

                                line_index = block.end;
                                block_index += 1;
                                previous_block_had_error = block.has_error;
                                while line_index < recent_lines.len()
                                    && recent_lines[line_index].trim().is_empty()
                                {
                                    line_index += 1;
                                }
                                continue;
                            }
                        }

                        let line = recent_lines[line_index];
                        if line.contains("docs-arch") || line.contains("feature-arch") {
                            println!("LINE DUMP: {:?}", line);
                        }
                        let wrapped_lines = get_terminal_wrapped_lines(line, wrap_cols);
                        for line_chunk in wrapped_lines {
                            let job = if line_chunk.starts_with("~ (") {
                                let mut job = egui::text::LayoutJob::default();
                                append_token(&mut job, &line_chunk, FG_MUTED, false);
                                job
                            } else {
                                output_line_job(&line_chunk, &session.cwd)
                            };
                            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(14, 0)).show(ui, |ui| {
                                ui.label(job);
                            });
                        }
                        previous_block_had_error = false;
                        line_index += 1;
                    }

                    if block_index > 0 {
                        fill_gap(ui, BLOCK_GAP_PAD_Y, Color32::TRANSPARENT);
                    }
                });
        });

    ui.painter().hline(
        sections.footer_rect.x_range(),
        sections.footer_rect.min.y,
        Stroke::new(1.0, Color32::from_gray(80)),
    );

    let mut footer_ui = ui.new_child(
        UiBuilder::new()
            .id_salt(("terminal-preview-footer", session_id))
            .max_rect(sections.footer_rect)
            .layout(Layout::top_down(Align::LEFT)),
    );
    footer_ui.scope_builder(
        UiBuilder::new()
            .max_rect(footer_layout.chips_rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
        let available_w = ui.available_width();
        // Overhead: 14px add_space + 16px btn-padding + 33px icon-spaces + 8px right = ~71px
        let path_max_chars = ((available_w - 71.0) / 7.0).floor().max(4.0) as usize;
        let show_branch = available_w > 180.0;

        ui.add_space(14.0);
        let short_cwd = truncate_path(&session.cwd, path_max_chars);
        let directory_button = terminal_directory_button(format!("     {}", short_cwd));
        let dir_resp = ui.add(directory_button);
        // Warp folder.svg icon painted over the chip's left side
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(dir_resp.rect.min.x + 11.0, dir_resp.rect.center().y),
            egui::vec2(12.0, 12.0),
        );
        ui.put(icon_rect, egui::Image::new(egui::include_image!("../assets/folder.svg")));

        if show_branch {
            if let Some((branch, _)) = shell::git_prompt_details(&session.cwd) {
                ui.add_space(TERMINAL_FOOTER_ROW_GAP);
                let branch_button = terminal_branch_button(format!("    {}", branch));
                let response = ui.add(branch_button);
                // Warp git-branch-02.svg icon (green)
                let b_icon_rect = egui::Rect::from_center_size(
                    egui::pos2(response.rect.min.x + 10.0, response.rect.center().y),
                    egui::vec2(12.0, 12.0),
                );
                ui.put(b_icon_rect, egui::Image::new(egui::include_image!("../assets/git-branch.svg")));
            }
        }
        },
    );

    let input_rect = footer_layout.input_rect;
    if input_rect.width() > 28.0 {
        footer_ui.painter().text(
            egui::pos2(input_rect.min.x + 16.0, input_rect.min.y + 17.0),
            egui::Align2::LEFT_CENTER,
            "Run commands",
            egui::FontId::monospace(14.0),
            FG_MUTED,
        );
    }

    footer_ui.scope_builder(UiBuilder::new().max_rect(input_rect), |ui| {
        ui.add_sized(
            [input_rect.width() - 28.0, footer_metrics.input_height],
            egui::TextEdit::singleline(&mut "".to_string())
                .frame(false)
                .margin(egui::Margin::symmetric(14, 10))
                .interactive(false)
                .font(egui::TextStyle::Monospace),
        );
    });

    // Click on any part of the inactive pane to focus it.
    let click_response = ui.interact(
        ui.max_rect(),
        ui.make_persistent_id(("terminal-pane-select", session_id)),
        egui::Sense::click(),
    );
    if click_response.clicked() {
        state.select_terminal_index(session_id);
        ui.memory_mut(|memory| memory.request_focus(egui::Id::new(("shell_input", session_id))));
    }
}

// handle_shortcuts moved to shortcuts.rs

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(FG_PRIMARY);
    visuals.panel_fill = BG_APP;
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.inactive.bg_fill = BG_PANEL_ALT;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.hovered.bg_fill = BG_PANEL;
    visuals.widgets.inactive.fg_stroke.color = FG_PRIMARY;
    visuals.window_fill = BG_APP;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.5, ACCENT_ALT);
    ctx.set_visuals(visuals);
}

#[cfg(test)]
fn trim_line(value: &str, max_chars: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return value.to_string();
    }

    chars[..max_chars.saturating_sub(1)]
        .iter()
        .collect::<String>()
        + "…"
}

fn truncate_path(value: &str, max_chars: usize) -> String {
    let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return "/".to_string();
    }

    if parts.len() == 1 {
        let first = parts[0];
        let chars: Vec<char> = first.chars().collect();
        if chars.len() > max_chars {
            let truncated: String = chars.into_iter().take(max_chars.saturating_sub(1)).collect();
            return format!("/{}…", truncated);
        }
        return format!("/{}", first);
    }

    let last = parts.last().unwrap();
    let chars: Vec<char> = last.chars().collect();
    let available_chars = max_chars.saturating_sub(2);
    
    if chars.len() > available_chars {
        let truncated: String = chars.into_iter().take(available_chars.saturating_sub(1)).collect();
        format!("…/{}…", truncated)
    } else {
        format!("…/{last}")
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "egui render state is split across app fields"
)]
fn render_focus_terminal(
    ui: &mut egui::Ui,
    session_id: usize,
    session: &core::app::TerminalState,
    state: &mut AppState,
    shell_input: &mut String,
    history_offset: &mut usize,
    selected_transcript_blocks: &mut BTreeSet<usize>,
    terminal_find_open: &mut bool,
    terminal_find_query: &mut String,
    terminal_find_case_sensitive: &mut bool,
    terminal_find_selected_only: &mut bool,
    terminal_find_active_result: &mut Option<usize>,
    directory_picker_open: &mut bool,
    directory_picker_query: &mut String,
    branch_picker_open: &mut bool,
    branch_picker_query: &mut String,
    action_tx: &mpsc::UnboundedSender<Action>,
) {
    let sections = terminal_pane_sections(ui.max_rect());
    let footer_layout = terminal_footer_layout(sections.footer_rect);
    let open_terminal_find = ui.input(|input| input.key_pressed(Key::F) && input.modifiers.command);
    if open_terminal_find {
        *terminal_find_open = true;
    }
    if *terminal_find_open && ui.input(|input| input.key_pressed(Key::Escape)) {
        *terminal_find_open = false;
        terminal_find_query.clear();
        *terminal_find_active_result = None;
    }

    let lines: Vec<&str> = session.lines.iter().map(String::as_str).collect();
    let search_scope =
        selected_search_scope(selected_transcript_blocks, *terminal_find_selected_only);
    let search_results = transcript_search_matches(
        &lines,
        &search_scope,
        terminal_find_query,
        *terminal_find_case_sensitive,
    );
    let footer_metrics = terminal_footer_metrics(sections.transcript_rect.height());
    if search_results.is_empty() {
        *terminal_find_active_result = None;
    } else if terminal_find_active_result.is_none_or(|index| index >= search_results.len()) {
        *terminal_find_active_result = Some(0);
    }

    ui.spacing_mut().item_spacing.y = 0.0;

        let mut transcript_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(("terminal-focus-transcript", session_id))
                .max_rect(sections.transcript_rect)
                .layout(Layout::top_down(Align::LEFT)),
        );
        let mut last_block_fill_color = Color32::TRANSPARENT;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(sections.transcript_rect.height())
            .stick_to_bottom(true)
            .show(&mut transcript_ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                
                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(0, 0))
                    .show(ui, |ui| {
                        let wrap_cols = wrap_columns_for_width((ui.available_width() - 28.0).max(1.0), 8.4);
                        let num_lines = lines
                            .iter()
                            .map(|line| crate::transcript::wrapped_row_count(line, wrap_cols))
                            .sum::<usize>() as f32;
                        let num_prompts = lines.iter().filter(|l| l.starts_with("$ ")).count() as f32;
                        let estimated_height = (num_lines * 14.5) + (num_prompts * 18.0) + 10.0;
                        let remaining_space = ui.available_height() - estimated_height;
                        if remaining_space > 0.0 {
                            ui.add_space(remaining_space);
                        }

                        let mut index = 0usize;
                        let mut previous_block_had_error = false;
                        let mut previous_block_selected = false;
                        let mut block_index = 0usize;
                        while index < lines.len() {
                            let line = lines[index];
                            let has_context_line = is_context_block_start(&lines, index);
                            if has_context_line || line.starts_with("$ ") {
                                let block_start = index;
                                let prompt_index = if has_context_line { index + 1 } else { index };
                                let mut block_end = prompt_index + 1;
                                while block_end < lines.len()
                                    && !lines[block_end].starts_with("$ ")
                                    && !is_context_block_start(&lines, block_end)
                                    && !is_legacy_context_block_start(&lines, block_end)
                                {
                                    block_end += 1;
                                }

                                let has_error = command_block_has_error(&lines, prompt_index);
                                let is_selected = selected_transcript_blocks.contains(&block_index);
                                let fill_color = transcript_block_fill_color(has_error, is_selected);
                                if block_index == 0 {
                                    draw_divider(ui, Color32::from_gray(60));
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, fill_color);
                                }

                                let render_separator =
                                    should_render_block_separator(previous_block_had_error, has_error);
                                if render_separator && block_index > 0 {
                                    let previous_fill_color = transcript_block_fill_color(
                                        previous_block_had_error,
                                        previous_block_selected,
                                    );
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, previous_fill_color);
                                    draw_divider(ui, Color32::from_gray(60));
                                    fill_gap(ui, BLOCK_GAP_PAD_Y, fill_color);
                                }

                                let block_width = ui.available_width();
                                let extend_error_block_to_bottom = should_extend_error_block_to_bottom(
                                    has_error,
                                    block_end == lines.len(),
                                );
                                let block_height = if extend_error_block_to_bottom {
                                    ui.available_height()
                                } else {
                                    0.0
                                };
                                let block_padding_y = if has_error { 16 } else { 8 };
                                let block_margin = egui::Margin {
                                    left: 14,
                                    right: 14,
                                    top: block_padding_y,
                                    bottom: block_padding_y,
                                };

                                let frame = Frame::new()
                                    .fill(fill_color)
                                    .stroke(Stroke::NONE)
                                    .corner_radius(0.0)
                                    .inner_margin(block_margin);

                                let frame_response = frame.show(ui, |ui| {
                                    ui.set_min_width((block_width - 28.0).max(1.0));
                                    if extend_error_block_to_bottom {
                                        ui.set_min_height(block_height);
                                    }
                                    for (line_index, block_line) in
                                        lines[block_start..block_end].iter().enumerate()
                                    {
                                        let absolute_line_index = block_start + line_index;
                                        let line_highlights: Vec<_> = search_results
                                            .iter()
                                            .filter(|result| result.line_index == absolute_line_index)
                                            .map(|result| result.range.clone())
                                            .collect();
                                        let active_highlight = terminal_find_active_result
                                            .and_then(|result_index| search_results.get(result_index))
                                            .filter(|result| result.line_index == absolute_line_index)
                                            .map(|result| result.range.clone());

                                        let wrapped_lines = get_terminal_wrapped_lines(block_line, wrap_cols);
                                        for line_chunk in wrapped_lines {
                                            let mut job = if is_command_context_line(&line_chunk) {
                                                context_line_job(&line_chunk)
                                            } else if line_chunk.starts_with("$ ") {
                                                command_line_job(&line_chunk, has_error)
                                            } else if line_chunk.starts_with("~ (") {
                                                let mut job = egui::text::LayoutJob::default();
                                                append_token(&mut job, &line_chunk, FG_MUTED, false);
                                                job
                                            } else {
                                                output_line_job(&line_chunk, &session.cwd)
                                            };
                                            apply_match_highlights(
                                                &mut job,
                                                &line_highlights,
                                                active_highlight.clone(),
                                            );
                                            let response = ui.label(job);
                                            if terminal_find_active_result
                                                .and_then(|result_index| search_results.get(result_index))
                                                .is_some_and(|result| result.line_index == absolute_line_index)
                                            {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                        }
                                    }
                                });

                                let click_response = ui.interact(
                                    frame_response.response.rect,
                                    ui.make_persistent_id(("transcript_block", block_index)),
                                    egui::Sense::click(),
                                );
                                if click_response.clicked() {
                                    update_selected_blocks(
                                        selected_transcript_blocks,
                                        block_index,
                                        ui.input(|input| input.modifiers.shift),
                                    );
                                }

                                previous_block_had_error = has_error;
                                previous_block_selected = is_selected;
                                last_block_fill_color = fill_color;
                                block_index += 1;
                                index = block_end;
                                while index < lines.len() && lines[index].trim().is_empty() {
                                    index += 1;
                                }
                                continue;
                            }

                            let line_highlights: Vec<_> = search_results
                                .iter()
                                .filter(|result| result.line_index == index)
                                .map(|result| result.range.clone())
                                .collect();
                            let active_highlight = terminal_find_active_result
                                .and_then(|result_index| search_results.get(result_index))
                                .filter(|result| result.line_index == index)
                                .map(|result| result.range.clone());

                            let wrapped_lines = get_terminal_wrapped_lines(line, wrap_cols);
                            for line_chunk in wrapped_lines {
                                let mut job = if line_chunk.starts_with("~ (") {
                                    let mut job = egui::text::LayoutJob::default();
                                    append_token(&mut job, &line_chunk, FG_MUTED, false);
                                    job
                                } else {
                                    output_line_job(&line_chunk, &session.cwd)
                                };
                                apply_match_highlights(&mut job, &line_highlights, active_highlight.clone());
                                egui::Frame::NONE.inner_margin(egui::Margin::symmetric(14, 0)).show(ui, |ui| {
                                    let response = ui.label(job);
                                    if terminal_find_active_result
                                        .and_then(|result_index| search_results.get(result_index))
                                        .is_some_and(|result| result.line_index == index)
                                    {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                });
                            }
                            previous_block_had_error = false;
                            previous_block_selected = false;
                            index += 1;
                        }

                        if block_index > 0 {
                            fill_gap(ui, BLOCK_GAP_PAD_Y, last_block_fill_color);
                        }
                    });
            });
        ui.painter().hline(
            sections.footer_rect.x_range(),
            sections.footer_rect.min.y,
            Stroke::new(1.0, Color32::from_gray(80)),
        );

        if *terminal_find_open {
            let find_id = ui.make_persistent_id("terminal_find_input");
            let find_input_has_focus = ui.memory(|memory| memory.has_focus(find_id));
            let navigate_forward = find_input_has_focus
                && ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::NONE, Key::Enter) && !input.modifiers.shift
                });
            let navigate_backward = find_input_has_focus
                && ui.input_mut(|input| input.consume_key(egui::Modifiers::SHIFT, Key::Enter));

            egui::Area::new(egui::Id::new("terminal_find_bar"))
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::RIGHT_TOP, [-18.0, 18.0])
                .show(ui.ctx(), |ui| {
                    let icon_find_button = |ui: &mut egui::Ui, label: &str| {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
                        if response.hovered() {
                            ui.painter()
                                .rect_filled(rect, 6.0, Color32::from_rgb(82, 82, 82));
                        }
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::monospace(16.0),
                            FG_PRIMARY,
                        );
                        response
                    };
                    let mut case_button_rect = None;
                    let mut case_button_hovered = false;
                    let mut selected_scope_button_rect = None;
                    let mut selected_scope_button_hovered = false;

                    Frame::new()
                        .fill(Color32::from_rgb(26, 26, 26))
                        .stroke(Stroke::NONE)
                        .corner_radius(7.0)
                        .inner_margin(egui::Margin::symmetric(6, 5))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.horizontal(|ui| {
                                Frame::new()
                                    .fill(Color32::from_rgb(19, 19, 19))
                                    .stroke(Stroke::new(1.0, Color32::from_gray(58)))
                                    .corner_radius(6.0)
                                    .inner_margin(egui::Margin::symmetric(8, 5))
                                    .show(ui, |ui| {
                                        ui.spacing_mut().item_spacing.x = 8.0;
                                        ui.horizontal(|ui| {
                                            let response = ui.add_sized(
                                                [240.0, 24.0],
                                                egui::TextEdit::singleline(terminal_find_query)
                                                    .id(find_id)
                                                    .frame(false)
                                                    .font(egui::TextStyle::Monospace),
                                            );
                                            if open_terminal_find {
                                                response.request_focus();
                                            }
                                            if response.changed() {
                                                *terminal_find_active_result = None;
                                            }
                                            if terminal_find_query.is_empty() {
                                                ui.painter().text(
                                                    egui::pos2(
                                                        response.rect.min.x + 4.0,
                                                        response.rect.center().y,
                                                    ),
                                                    egui::Align2::LEFT_CENTER,
                                                    "Find",
                                                    egui::FontId::monospace(13.0),
                                                    Color32::from_rgb(105, 105, 105),
                                                );
                                            }

                                            let aa_button = egui::Button::new(
                                                RichText::new("Aa").monospace().size(16.0).color(
                                                    if *terminal_find_case_sensitive {
                                                        BG_APP
                                                    } else {
                                                        FG_PRIMARY
                                                    },
                                                ),
                                            )
                                            .fill(if *terminal_find_case_sensitive {
                                                Color32::from_rgb(26, 174, 232)
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .stroke(Stroke::new(1.0, Color32::from_gray(88)))
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(34.0, 26.0));
                                            let aa_response = ui.add(aa_button);
                                            case_button_rect = Some(aa_response.rect);
                                            case_button_hovered = aa_response.hovered();
                                            if aa_response.clicked() {
                                                *terminal_find_case_sensitive =
                                                    !*terminal_find_case_sensitive;
                                                *terminal_find_active_result = None;
                                            }

                                            let selected_scope_button = egui::Button::new(
                                                RichText::new("⛶").monospace().size(16.0).color(
                                                    if *terminal_find_selected_only {
                                                        BG_APP
                                                    } else {
                                                        FG_PRIMARY
                                                    },
                                                ),
                                            )
                                            .fill(if *terminal_find_selected_only {
                                                Color32::from_rgb(26, 174, 232)
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .stroke(Stroke::new(1.0, Color32::from_gray(88)))
                                            .corner_radius(6.0)
                                            .min_size(egui::vec2(30.0, 26.0));
                                            let selected_scope_response =
                                                ui.add(selected_scope_button);
                                            selected_scope_button_rect =
                                                Some(selected_scope_response.rect);
                                            selected_scope_button_hovered =
                                                selected_scope_response.hovered();
                                            if selected_scope_response.clicked() {
                                                *terminal_find_selected_only =
                                                    !*terminal_find_selected_only;
                                                *terminal_find_active_result = None;
                                            }
                                        });
                                    });

                                let result_label = if search_results.is_empty() {
                                    "0/0".to_string()
                                } else {
                                    format!(
                                        "{}/{}",
                                        terminal_find_active_result.unwrap_or(0) + 1,
                                        search_results.len()
                                    )
                                };
                                ui.label(RichText::new(result_label).monospace().color(FG_MUTED));

                                if icon_find_button(ui, "↓").clicked() || navigate_forward {
                                    *terminal_find_active_result = next_search_index(
                                        *terminal_find_active_result,
                                        search_results.len(),
                                        false,
                                    );
                                }
                                if icon_find_button(ui, "↑").clicked() || navigate_backward {
                                    *terminal_find_active_result = next_search_index(
                                        *terminal_find_active_result,
                                        search_results.len(),
                                        true,
                                    );
                                }
                                if icon_find_button(ui, "×").clicked() {
                                    *terminal_find_open = false;
                                    terminal_find_query.clear();
                                    *terminal_find_active_result = None;
                                }
                            });
                        });

                    if case_button_hovered {
                        if let Some(rect) = case_button_rect {
                            show_button_tooltip(
                                ui.ctx(),
                                "terminal_find_case_tooltip",
                                rect,
                                "Case sensitive search",
                            );
                        }
                    }
                    if selected_scope_button_hovered {
                        if let Some(rect) = selected_scope_button_rect {
                            show_button_tooltip(
                                ui.ctx(),
                                "terminal_find_selected_scope_tooltip",
                                rect,
                                "Find in selected block",
                            );
                        }
                    }
                });
        }

        let mut footer_ui = ui.new_child(
            UiBuilder::new()
                .id_salt(("terminal-focus-footer", session_id))
                .max_rect(sections.footer_rect)
                .layout(Layout::top_down(Align::LEFT)),
        );
        let mut directory_button_rect = None;
        let mut branch_button_rect = None;
        let mut directory_button_hovered = false;
        let mut branch_button_hovered = false;
        footer_ui.scope_builder(
            UiBuilder::new()
                .max_rect(footer_layout.chips_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
            let available_w = ui.available_width();
            let path_max_chars = ((available_w - 71.0) / 7.0).floor().max(4.0) as usize;
            let show_branch = available_w > 180.0;

            ui.add_space(14.0);
            let short_cwd = truncate_path(&session.cwd, path_max_chars);
            let directory_button = terminal_directory_button(format!("     {}", short_cwd));
            let directory_response = ui.add(directory_button);
            // Warp folder.svg icon — painted over chip left side
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(directory_response.rect.min.x + 11.0, directory_response.rect.center().y),
                egui::vec2(12.0, 12.0),
            );
            ui.put(icon_rect, egui::Image::new(egui::include_image!("../assets/folder.svg")));
            directory_button_rect = Some(directory_response.rect);
            directory_button_hovered = directory_response.hovered();
            if directory_response.clicked() {
                *directory_picker_open = true;
                *branch_picker_open = false;
                directory_picker_query.clear();
            }

            if show_branch {
                if let Some((branch, _)) = shell::git_prompt_details(&session.cwd) {
                    ui.add_space(TERMINAL_FOOTER_ROW_GAP);
                    let branch_button = terminal_branch_button(format!("    {}", branch));
                    let response = ui.add(branch_button);
                    branch_button_rect = Some(response.rect);
                    branch_button_hovered = response.hovered();
                    // Warp git-branch-02.svg icon (green)
                    let b_icon_rect = egui::Rect::from_center_size(
                        egui::pos2(response.rect.min.x + 10.0, response.rect.center().y),
                        egui::vec2(12.0, 12.0),
                    );
                    ui.put(b_icon_rect, egui::Image::new(egui::include_image!("../assets/git-branch.svg")));
                    if response.clicked() {
                        *branch_picker_open = true;
                        *directory_picker_open = false;
                        branch_picker_query.clear();
                    }
                }
            }
            },
        );

        if directory_button_hovered {
            if let Some(rect) = directory_button_rect {
                show_button_tooltip(
                    ui.ctx(),
                    "directory_button_tooltip",
                    rect,
                    "Change working directory",
                );
            }
        }
        if branch_button_hovered {
            if let Some(rect) = branch_button_rect {
                show_button_tooltip(ui.ctx(), "branch_button_tooltip", rect, "Change git branch");
            }
        }
        if *directory_picker_open {
            let mut close_picker = false;
            if ui.ctx().input(|input| input.key_pressed(Key::Escape)) {
                close_picker = true;
            }

            let directory_options = directory_picker_options(&session.cwd, directory_picker_query);
            let directory_picker_size =
                egui::vec2(400.0, picker_height(directory_options.len(), 28.0, 8));
            let directory_picker_pos = directory_button_rect
                .map(|rect| egui::pos2(rect.min.x, rect.min.y - directory_picker_size.y - 16.0))
                .unwrap_or_else(|| egui::pos2(14.0, 0.0));
            let directory_popup_rect = egui::Window::new("directory_picker")
                .title_bar(false)
                .resizable(false)
                .fixed_size(directory_picker_size)
                .fixed_pos(directory_picker_pos)
                .show(ui.ctx(), |ui| {
                    let search_id = ui.make_persistent_id("directory_picker_search");
                    let search = Frame::new()
                        .fill(BG_APP)
                        .stroke(Stroke::NONE)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width(), 28.0],
                                egui::TextEdit::singleline(directory_picker_query)
                                    .id(search_id)
                                    .hint_text("Search directories...")
                                    .frame(false),
                            )
                        })
                        .inner;
                    if !search.has_focus() {
                        search.request_focus();
                    }

                    ScrollArea::vertical()
                        .max_height(directory_picker_size.y - 52.0)
                        .show(ui, |ui| {
                            for option in &directory_options {
                                let prefix = if option.is_parent { "↑" } else { "🗀" };
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 28.0),
                                    egui::Sense::click(),
                                );
                                if response.hovered() {
                                    ui.painter().rect_filled(rect, 0.0, PICKER_HOVER);
                                }

                                let text_color = if response.hovered() {
                                    BG_APP
                                } else if option.is_parent {
                                    FG_MUTED
                                } else {
                                    FG_PRIMARY
                                };
                                ui.painter().text(
                                    egui::pos2(rect.min.x + 10.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    prefix,
                                    egui::FontId::monospace(13.0),
                                    text_color,
                                );
                                ui.painter().text(
                                    egui::pos2(rect.min.x + 34.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &option.label,
                                    egui::FontId::monospace(12.0),
                                    text_color,
                                );

                                if response.clicked() {
                                    let command =
                                        format!("cd {}", shell_quote_path(&option.target_path));
                                    if submit_shell_command(
                                        state,
                                        action_tx.clone(),
                                        history_offset,
                                        command,
                                    ) {
                                        shell_input.clear();
                                        close_picker = true;
                                    }
                                }
                            }
                        });
                })
                .map(|inner| inner.response.rect);
            if ui.ctx().input(|input| input.pointer.any_pressed()) {
                if let Some(pointer_pos) = ui.ctx().input(|input| input.pointer.interact_pos()) {
                    let clicked_directory_button =
                        directory_button_rect.is_some_and(|rect| rect.contains(pointer_pos));
                    let clicked_popup =
                        directory_popup_rect.is_some_and(|rect| rect.contains(pointer_pos));
                    if !clicked_directory_button && !clicked_popup {
                        close_picker = true;
                    }
                }
            }
            if close_picker {
                *directory_picker_open = false;
                directory_picker_query.clear();
            }
        }

        if *branch_picker_open {
            let mut close_picker = false;
            if ui.ctx().input(|input| input.key_pressed(Key::Escape)) {
                close_picker = true;
            }

            let branch_options = branch_picker_options(&session.cwd, branch_picker_query);
            let branch_picker_pos = branch_button_rect
                .map(|rect| {
                    let size = egui::vec2(360.0, picker_height(branch_options.len(), 28.0, 6));
                    egui::pos2(rect.min.x, rect.min.y - size.y - 16.0)
                })
                .unwrap_or_else(|| egui::pos2(448.0, 0.0));
            let branch_picker_size =
                egui::vec2(360.0, picker_height(branch_options.len(), 28.0, 6));
            let branch_popup_rect = egui::Window::new("branch_picker")
                .title_bar(false)
                .resizable(false)
                .fixed_size(branch_picker_size)
                .fixed_pos(branch_picker_pos)
                .show(ui.ctx(), |ui| {
                    let search_id = ui.make_persistent_id("branch_picker_search");
                    let search = Frame::new()
                        .fill(BG_APP)
                        .stroke(Stroke::NONE)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.add_sized(
                                [ui.available_width(), 28.0],
                                egui::TextEdit::singleline(branch_picker_query)
                                    .id(search_id)
                                    .hint_text("Search branches...")
                                    .frame(false),
                            )
                        })
                        .inner;
                    if !search.has_focus() {
                        search.request_focus();
                    }

                    ScrollArea::vertical()
                        .max_height(branch_picker_size.y - 52.0)
                        .show(ui, |ui| {
                            for option in &branch_options {
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 28.0),
                                    egui::Sense::click(),
                                );
                                if response.hovered() {
                                    ui.painter().rect_filled(rect, 0.0, PICKER_HOVER);
                                }

                                let text_color = if response.hovered() {
                                    BG_APP
                                } else {
                                    FG_PRIMARY
                                };
                                draw_branch_icon(
                                    ui.painter(),
                                    rect.min.x + 10.0,
                                    rect.center().y,
                                    if response.hovered() {
                                        BG_APP
                                    } else {
                                        BRANCH_GREEN
                                    },
                                );
                                ui.painter().text(
                                    egui::pos2(rect.min.x + 38.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &option.label,
                                    egui::FontId::monospace(12.0),
                                    if option.is_current && !response.hovered() {
                                        BRANCH_GREEN
                                    } else {
                                        text_color
                                    },
                                );

                                if response.clicked() {
                                    let command = format!("git checkout {}", option.label);
                                    if submit_shell_command(
                                        state,
                                        action_tx.clone(),
                                        history_offset,
                                        command,
                                    ) {
                                        shell_input.clear();
                                        close_picker = true;
                                    }
                                }
                            }
                        });
                })
                .map(|inner| inner.response.rect);
            if ui.ctx().input(|input| input.pointer.any_pressed()) {
                if let Some(pointer_pos) = ui.ctx().input(|input| input.pointer.interact_pos()) {
                    let clicked_branch_button =
                        branch_button_rect.is_some_and(|rect| rect.contains(pointer_pos));
                    let clicked_popup =
                        branch_popup_rect.is_some_and(|rect| rect.contains(pointer_pos));
                    if !clicked_branch_button && !clicked_popup {
                        close_picker = true;
                    }
                }
            }
            if close_picker {
                *branch_picker_open = false;
                branch_picker_query.clear();
            }
        }

        let input_id = egui::Id::new(("shell_input", session_id));

        let mut rect = footer_layout.input_rect;
        rect.min.x += 14.0;

        let suggestion = shell::command_suggestion(
            &session.cwd,
            &session.history,
            state.terminal_directory_history(),
            shell_input,
        );
        let suggestion_suffix = terminal_suggestion_suffix(shell_input, suggestion.as_deref());
        let completion_requested = !*directory_picker_open
            && !*branch_picker_open
            && (ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Tab))
                || ui.input(|i| i.key_pressed(Key::ArrowRight)));
        let enter_pressed = ui.input(|input| {
            input.key_pressed(Key::Enter)
                && !input.modifiers.shift
                && !input.modifiers.command
                && !input.modifiers.ctrl
                && !input.modifiers.alt
        });

        if shell_input.is_empty() {
            footer_ui.painter().text(
                egui::pos2(rect.min.x + 2.0, rect.min.y + 17.0),
                egui::Align2::LEFT_CENTER,
                "Run commands",
                egui::FontId::monospace(14.0),
                FG_MUTED,
            );
        } else if let Some(ref suffix) = suggestion_suffix {
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let prefix_width = ui
                .painter()
                .layout_no_wrap(shell_input.clone(), font_id, Color32::TRANSPARENT)
                .size()
                .x;
            let suffix_x = rect.min.x + 2.0 + prefix_width;
            let suffix_center_y = rect.min.y + 17.0;
            footer_ui.painter().text(
                egui::pos2(suffix_x, suffix_center_y),
                egui::Align2::LEFT_CENTER,
                suffix,
                egui::FontId::monospace(13.0),
                Color32::from_gray(80),
            );

            let suffix_width = ui
                .painter()
                .layout_no_wrap(
                    suffix.to_string(),
                    egui::FontId::monospace(13.0),
                    Color32::TRANSPARENT,
                )
                .size()
                .x;
            let arrow_rect = egui::Rect::from_center_size(
                egui::pos2(suffix_x + suffix_width + 24.0, suffix_center_y),
                egui::vec2(36.0, 24.0),
            );
            footer_ui.painter().rect_stroke(
                arrow_rect,
                4.0,
                Stroke::new(1.0, Color32::from_gray(50)),
                egui::StrokeKind::Inside,
            );
            footer_ui.painter().text(
                arrow_rect.center(),
                egui::Align2::CENTER_CENTER,
                "→▾",
                egui::FontId::monospace(10.0),
                FG_MUTED,
            );
        }

        let response = footer_ui
            .scope_builder(UiBuilder::new().max_rect(footer_layout.input_rect), |ui| {
                ui.add_sized(
                    [footer_layout.input_rect.width() - 28.0, footer_metrics.input_height],
                    egui::TextEdit::singleline(shell_input)
                        .id(input_id)
                        .frame(false)
                        .margin(egui::Margin::symmetric(14, 10))
                        .lock_focus(true)
                        .font(egui::TextStyle::Monospace),
                )
            })
            .inner;

        if response.has_focus() {
            let history = &session.history;
            if !history.is_empty() {
                if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                    if *history_offset < history.len() {
                        *history_offset += 1;
                    }
                    if let Some(entry) = history_entry_for_offset(history, *history_offset) {
                        *shell_input = entry;
                        if let Some(mut text_state) = egui::TextEdit::load_state(ui.ctx(), input_id)
                        {
                            let ccursor = egui::text::CCursor::new(shell_input.chars().count());
                            text_state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                            text_state.store(ui.ctx(), input_id);
                        }
                    }
                } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                    if *history_offset > 1 {
                        *history_offset -= 1;
                        if let Some(entry) = history_entry_for_offset(history, *history_offset) {
                            *shell_input = entry;
                        }
                    } else if *history_offset == 1 {
                        *history_offset = 0;
                        shell_input.clear();
                    }
                    if let Some(mut text_state) = egui::TextEdit::load_state(ui.ctx(), input_id) {
                        let ccursor = egui::text::CCursor::new(shell_input.chars().count());
                        text_state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                        text_state.store(ui.ctx(), input_id);
                    }
                }
            }
        }

        if let Some(sugg) = suggestion {
            if completion_requested {
                *shell_input = sugg;
                ui.memory_mut(|memory| memory.request_focus(input_id));
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), input_id) {
                    let ccursor = egui::text::CCursor::new(shell_input.chars().count());
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), input_id);
                }
            }
        }

        if !*directory_picker_open
            && !*branch_picker_open
            && !*terminal_find_open
            && !response.has_focus()
            && shell_input.is_empty()
        {
            ui.memory_mut(|memory| memory.request_focus(input_id));
        }
        let submit_requested = shell_input_should_submit(
            *directory_picker_open,
            *branch_picker_open,
            response.has_focus(),
            response.lost_focus(),
            enter_pressed,
        );
        if submit_requested && !shell_input.trim().is_empty() {
            let command = shell_input.trim().to_string();
            if submit_shell_command(state, action_tx.clone(), history_offset, command) {
                shell_input.clear();
                response.request_focus();
            }
        }

}

fn spawn_core_runtime(state: Arc<RwLock<AppState>>) -> mpsc::UnboundedSender<Action> {
    let (tx, mut rx) = mpsc::unbounded_channel::<mpsc::UnboundedSender<Action>>();

    thread::spawn(move || {
        let Ok(runtime) = Builder::new_multi_thread().enable_all().build() else {
            eprintln!("failed to start desktop runtime");
            return;
        };

        runtime.block_on(async move {
            // Give the engine an action_tx so it can be returned via our proxy channel
            let action_tx = CoreEngine::spawn_background(state.clone());
            let _ = tx.send(action_tx.clone());

            // 1 default terminal initially
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
            let _ = action_tx.send(Action::SpawnTerminal { cwd });

            // Spawn LAN web server (REST + WebSocket) on a background task.
            let web_state = state.clone();
            let web_port: u16 = std::env::var("VIEW_WEB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(23779);
            tokio::spawn(async move {
                if let Err(err) = web::start(web_state, web_port).await {
                    eprintln!("web server error: {err}");
                }
            });

            // Keep runtime alive
            std::future::pending::<()>().await;
        });
    });

    // Wait for the runtime to boot up and return the actual action_tx
    match rx.blocking_recv() {
        Some(action_tx) => action_tx,
        None => {
            eprintln!("failed to receive action_tx from background thread");
            let (fallback_tx, _fallback_rx) = mpsc::unbounded_channel();
            fallback_tx
        }
    }
}

fn screenshot_target() -> Option<PathBuf> {
    std::env::var("VIEW_DESKTOP_SCREENSHOT_TO")
        .or_else(|_| std::env::var("EFRAME_SCREENSHOT_TO"))
        .ok()
        .map(PathBuf::from)
}

fn debug_log_path() -> Option<PathBuf> {
    std::env::var("VIEW_DESKTOP_DEBUG_LOG")
        .ok()
        .map(PathBuf::from)
}

fn debug_log(message: String) {
    let Some(path) = debug_log_path() else {
        return;
    };

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn save_color_image(path: &PathBuf, image: &egui::ColorImage) -> anyhow::Result<()> {
    let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
    for color in &image.pixels {
        let [r, g, b, a] = color.to_array();
        rgba.extend_from_slice(&[r, g, b, a]);
    }

    let Some(buffer) =
        ImageBuffer::<Rgba<u8>, _>::from_raw(image.size[0] as u32, image.size[1] as u32, rgba)
    else {
        return Err(anyhow::anyhow!("Failed to build RGBA image buffer"));
    };

    buffer.save(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use eframe::egui;

    use super::{
        command_line_job, find_match_ranges, next_search_index, screenshot_target,
        selected_search_scope, shell_input_should_submit, terminal_footer_metrics,
        terminal_suggestion_suffix, terminal_transcript_blocks, terminal_size_for_content_rect,
        transcript_block_fill_color, transcript_search_matches, trim_line, truncate_path,
        update_selected_blocks, ERROR_COMMAND_TEXT, ERROR_PANEL_BG, SELECTED_PANEL_BG,
    };
    use crate::shell::{
        command_suggestion, directory_picker_options, format_command_context_line,
        history_entry_for_offset, shell_quote_path,
    };
    use crate::transcript::{
        build_physical_rows, command_block_has_error, command_clears_transcript,
        is_context_block_start, is_legacy_context_block_start,
        selection_text_from_physical_rows, should_extend_error_block_to_bottom,
        should_render_block_separator, wrap_columns_for_width, wrapped_row_count,
    };

    #[test]
    fn string_helpers_should_trim_without_panicking() {
        assert_eq!(trim_line("abcdef", 4), "abc…");
        assert_eq!(truncate_path("/a/very/long/path/file.rs", 10), "…/file.rs");
    }

    #[test]
    fn screenshot_target_should_prefer_desktop_specific_env() {
        std::env::set_var("VIEW_DESKTOP_SCREENSHOT_TO", "/tmp/a.png");
        std::env::set_var("EFRAME_SCREENSHOT_TO", "/tmp/b.png");

        assert_eq!(
            screenshot_target().as_deref(),
            Some(std::path::Path::new("/tmp/a.png"))
        );

        std::env::remove_var("VIEW_DESKTOP_SCREENSHOT_TO");
        std::env::remove_var("EFRAME_SCREENSHOT_TO");
    }

    #[test]
    fn terminal_suggestion_suffix_should_render_only_remaining_text() {
        assert_eq!(
            terminal_suggestion_suffix("cd ", Some("cd ..")),
            Some("..".to_string())
        );
    }

    #[test]
    fn terminal_suggestion_suffix_should_ignore_exact_matches() {
        assert_eq!(terminal_suggestion_suffix("cd ..", Some("cd ..")), None);
    }

    #[test]
    fn command_clears_transcript_should_match_clear_aliases() {
        assert!(command_clears_transcript("clear"));
        assert!(command_clears_transcript(" cls "));
        assert!(!command_clears_transcript("clear now"));
    }

    #[test]
    fn command_block_has_error_should_detect_failed_command_output() {
        let lines = vec![
            "$ cd vivisual_interception_event_window",
            "~ (0.0006s)",
            "cd: no such file or directory: vivisual_interception_event_window",
            "$ cd visual_interception_event_window",
        ];

        assert!(command_block_has_error(&lines, 0));
        assert!(!command_block_has_error(&lines, 3));
    }

    #[test]
    fn should_render_block_separator_should_render_between_all_transcript_blocks() {
        assert!(should_render_block_separator(false, false));
        assert!(should_render_block_separator(true, false));
        assert!(should_render_block_separator(false, true));
    }

    #[test]
    fn should_extend_error_block_to_bottom_should_only_apply_to_final_error_block() {
        assert!(should_extend_error_block_to_bottom(true, true));
        assert!(!should_extend_error_block_to_bottom(true, false));
        assert!(!should_extend_error_block_to_bottom(false, true));
    }

    #[test]
    fn command_line_job_should_render_failed_commands_in_error_color() {
        let job = command_line_job("$ ls /missing", true);

        assert!(job
            .sections
            .iter()
            .all(|section| section.format.color == ERROR_COMMAND_TEXT));
    }

    #[test]
    fn transcript_block_fill_color_should_prefer_selection_without_changing_error_text_rules() {
        assert_eq!(transcript_block_fill_color(true, true), SELECTED_PANEL_BG);
        assert_eq!(transcript_block_fill_color(true, false), ERROR_PANEL_BG);
    }

    #[test]
    fn find_match_ranges_should_support_case_sensitive_toggle() {
        assert_eq!(find_match_ranges("ls0 LS0", "ls0", false).len(), 2);
        assert_eq!(find_match_ranges("ls0 LS0", "ls0", true).len(), 1);
    }

    #[test]
    fn transcript_search_matches_should_scope_to_selected_blocks_only() {
        let lines = vec![
            "/tmp/project git:(main)",
            "$ ls0",
            "zsh: command not found: ls0",
            "/tmp/project git:(main)",
            "$ echo ls0",
            "ls0",
        ];

        let all_matches = transcript_search_matches(&lines, &BTreeSet::new(), "ls0", false);
        assert_eq!(all_matches.len(), 4);

        let selected_matches =
            transcript_search_matches(&lines, &BTreeSet::from([1usize]), "ls0", false);
        assert_eq!(selected_matches.len(), 2);
        assert!(selected_matches
            .iter()
            .all(|result| result.block_index == Some(1)));
    }

    #[test]
    fn next_search_index_should_wrap_in_both_directions() {
        assert_eq!(next_search_index(None, 3, false), Some(0));
        assert_eq!(next_search_index(Some(2), 3, false), Some(0));
        assert_eq!(next_search_index(None, 3, true), Some(2));
        assert_eq!(next_search_index(Some(0), 3, true), Some(2));
        assert_eq!(next_search_index(Some(1), 0, false), None);
    }

    #[test]
    fn selected_search_scope_should_only_apply_when_toggle_is_enabled() {
        let selected = BTreeSet::from([1usize, 3usize]);

        assert!(selected_search_scope(&selected, false).is_empty());
        assert_eq!(selected_search_scope(&selected, true), selected);
    }

    #[test]
    fn shell_input_should_submit_when_enter_makes_text_edit_lose_focus() {
        assert!(shell_input_should_submit(false, false, false, true, true));
        assert!(shell_input_should_submit(false, false, true, false, true));
        assert!(!shell_input_should_submit(true, false, false, true, true));
        assert!(!shell_input_should_submit(false, true, false, true, true));
        assert!(!shell_input_should_submit(false, false, false, false, true));
    }

    #[test]
    fn terminal_size_for_content_rect_should_clamp_to_reasonable_bounds() {
        let tiny = terminal_size_for_content_rect(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(200.0, 120.0),
        ));
        let large = terminal_size_for_content_rect(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2400.0, 1800.0),
        ));

        assert_eq!(tiny.cols, 40);
        assert_eq!(tiny.rows, 10);
        assert!(large.cols <= 320);
        assert!(large.rows <= 160);
        assert!(large.cols > tiny.cols);
        assert!(large.rows > tiny.rows);
    }

    #[test]
    fn terminal_footer_metrics_should_keep_preview_and_active_input_heights_aligned() {
        let metrics = terminal_footer_metrics(420.0);

        assert_eq!(metrics.input_height, 44.0);
        assert_eq!(metrics.transcript_height, 294.0);
    }

    #[test]
    fn terminal_transcript_blocks_should_detect_each_command_group() {
        let lines = vec![
            "/tmp/project git:(main)",
            "$ ls",
            "Cargo.toml",
            "/tmp/project git:(main)",
            "$ bad",
            "zsh: command not found: bad",
            "plain trailing line",
        ];

        let blocks = terminal_transcript_blocks(&lines);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].start, 0);
        assert_eq!(blocks[0].end, 3);
        assert!(!blocks[0].has_error);
        assert_eq!(blocks[1].start, 3);
        assert_eq!(blocks[1].end, 7);
        assert!(blocks[1].has_error);
    }

    #[test]
    fn wrapped_row_count_should_track_physical_rows_without_changing_logical_line_count() {
        let line = "abcdefghijklmnopqrstuvwxyz";
        assert_eq!(wrapped_row_count(line, 10), 3);
        assert_eq!(wrapped_row_count(line, 26), 1);
    }

    #[test]
    fn selection_text_from_physical_rows_should_preserve_single_logical_line() {
        let lines = vec!["abcdefghijklmnopqrstuvwxyz"];
        let rows = build_physical_rows(&lines, 10);

        assert_eq!(
            selection_text_from_physical_rows(&lines, &rows, 0..=2),
            "abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn selection_text_from_physical_rows_should_insert_newlines_between_logical_lines_only() {
        let lines = vec!["abcdefghijklmno", "pqrstuvwxyz"];
        let rows = build_physical_rows(&lines, 5);

        assert_eq!(
            selection_text_from_physical_rows(&lines, &rows, 1..=4),
            "fghijklmno\npqrstuvwxy"
        );
    }

    #[test]
    fn wrap_columns_for_width_should_never_return_zero() {
        assert_eq!(wrap_columns_for_width(0.0, 8.4), 1);
        assert_eq!(wrap_columns_for_width(16.8, 8.4), 2);
    }

    #[test]
    fn format_command_context_line_should_include_git_details_when_present() {
        assert_eq!(
            format_command_context_line(
                "/tmp/project",
                Some("main"),
                Some("4 files changed, 10 insertions(+)")
            ),
            "/tmp/project git:(main) 4 files changed, 10 insertions(+)"
        );
    }

    #[test]
    fn context_block_detection_should_handle_current_and_legacy_layouts() {
        let current = vec!["/tmp/project git:(main)", "$ ls"];
        let legacy = vec!["", "/tmp/project git:(main)", "$ ls"];

        assert!(is_context_block_start(&current, 0));
        assert!(is_legacy_context_block_start(&legacy, 0));
    }

    #[test]
    fn history_entry_for_offset_should_return_most_recent_first() {
        let history = std::collections::VecDeque::from(vec!["ls".to_string(), "cd ..".to_string()]);

        assert_eq!(
            history_entry_for_offset(&history, 1).as_deref(),
            Some("cd ..")
        );
        assert_eq!(history_entry_for_offset(&history, 2).as_deref(), Some("ls"));
    }

    #[test]
    fn shell_quote_path_should_escape_single_quotes() {
        assert_eq!(shell_quote_path("/tmp/it's-here"), "'/tmp/it'\\''s-here'");
    }

    #[test]
    fn directory_picker_options_should_include_parent_and_filter_children() {
        let root = std::env::temp_dir().join(format!(
            "_cmd-picker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("alpha")).expect("alpha");
        std::fs::create_dir_all(root.join("beta")).expect("beta");

        let options = directory_picker_options(root.to_str().expect("utf8"), "alp");

        assert!(options.iter().any(|option| option.is_parent));
        assert!(options.iter().any(|option| option.label == "alpha"));
        assert!(!options.iter().any(|option| option.label == "beta"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn command_suggestion_should_complete_cd_from_directories() {
        let root = std::env::temp_dir().join(format!(
            "_cmd-suggest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("visual_interception_event_window")).expect("dir");

        let suggestion = command_suggestion(
            root.to_str().expect("utf8"),
            &std::collections::VecDeque::new(),
            &std::collections::VecDeque::new(),
            "cd vi",
        );

        assert_eq!(
            suggestion.as_deref(),
            Some("cd visual_interception_event_window")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn command_suggestion_should_prefer_local_directory_over_global_jump_match() {
        let root = std::env::temp_dir().join(format!(
            "_cmd-local-dir-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("crates")).expect("crates");

        let history = std::collections::VecDeque::from(vec![
            "cd /Users/me/worktrees/crates-long-path".to_string(),
        ]);
        let directory_history = std::collections::VecDeque::from(vec![
            "/Users/me/worktrees/crates-long-path".to_string(),
        ]);

        let suggestion = command_suggestion(
            root.to_str().expect("utf8"),
            &history,
            &directory_history,
            "cd cr",
        );

        assert_eq!(suggestion.as_deref(), Some("cd crates"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn command_suggestion_should_prefer_recent_cd_history_for_global_directory_jump() {
        let root = std::env::temp_dir().join(format!(
            "_cmd-global-jump-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root");

        let history = std::collections::VecDeque::from(vec![
            "cd /tmp/alpha-project".to_string(),
            "cd /tmp/visual_interception_event_window".to_string(),
        ]);

        let directory_history = std::collections::VecDeque::from(vec![
            "/tmp/alpha-project".to_string(),
            "/tmp/visual_interception_event_window".to_string(),
        ]);
        let suggestion = command_suggestion(
            root.to_str().expect("utf8"),
            &history,
            &directory_history,
            "cd vi",
        );

        assert_eq!(
            suggestion.as_deref(),
            Some("cd /tmp/visual_interception_event_window")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn command_suggestion_should_match_cd_history_by_path_fragment() {
        let history = std::collections::VecDeque::from(vec![
            "cd /Users/me/src/alpha".to_string(),
            "cd /Users/me/worktrees/visual_interception_event_window".to_string(),
        ]);

        let directory_history = std::collections::VecDeque::from(vec![
            "/Users/me/src/alpha".to_string(),
            "/Users/me/worktrees/visual_interception_event_window".to_string(),
        ]);
        let suggestion = command_suggestion("/tmp", &history, &directory_history, "cd work");

        assert_eq!(
            suggestion.as_deref(),
            Some("cd /Users/me/worktrees/visual_interception_event_window")
        );
    }

    #[test]
    fn update_selected_blocks_should_toggle_and_support_shift_multi_select() {
        let mut selected = BTreeSet::new();
        update_selected_blocks(&mut selected, 2, false);
        assert_eq!(selected, BTreeSet::from([2]));

        update_selected_blocks(&mut selected, 2, false);
        assert!(selected.is_empty());

        update_selected_blocks(&mut selected, 2, false);
        update_selected_blocks(&mut selected, 3, true);
        assert_eq!(selected, BTreeSet::from([2, 3]));
    }
}
