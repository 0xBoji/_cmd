//! Terminal transcript rendering for VIEW Desktop.
//!
//! Handles the scrollable output area: command blocks, error highlighting,
//! context lines (cwd + git), and block separators. Kept separate from
//! input handling and shell plumbing so each can evolve independently.

use eframe::egui::{self, Align, Color32, Frame, Layout, RichText, ScrollArea, Stroke};
use view_core::app::AppState;

// Local color constants — mirrors desktop_app palette
const BG_APP: Color32 = Color32::from_rgb(10, 11, 14);
const FG_PRIMARY: Color32 = Color32::from_rgb(234, 238, 255);
const FG_MUTED: Color32 = Color32::from_rgb(145, 154, 188);
const ERROR_PANEL_BG: Color32 = Color32::from_rgb(96, 40, 40);
const ERROR_TEXT: Color32 = Color32::from_rgb(255, 228, 228);

// ── Helpers re-exported for tests ─────────────────────────────────────────────

pub fn command_clears_transcript(command: &str) -> bool {
    matches!(command.trim(), "clear" | "cls")
}

pub fn is_error_output_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("command not found")
        || lower.contains("no such file or directory")
        || lower.contains("error:")
        || lower.contains("permission denied")
        || lower.starts_with("zsh: ")
}

pub fn command_block_has_error(lines: &[&str], prompt_index: usize) -> bool {
    lines
        .iter()
        .skip(prompt_index + 1)
        .take_while(|line| !line.starts_with("$ "))
        .any(|line| is_error_output_line(line))
}

pub fn should_render_block_separator(
    previous_block_had_error: bool,
    current_block_has_error: bool,
) -> bool {
    !current_block_has_error && !previous_block_had_error
}

pub fn should_extend_error_block_to_bottom(has_error: bool, is_last_block: bool) -> bool {
    has_error && is_last_block
}

pub fn is_command_context_line(line: &str) -> bool {
    line.starts_with('/')
}

pub fn is_context_block_start(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| is_command_context_line(line))
        && lines
            .get(index + 1)
            .is_some_and(|next| next.starts_with("$ "))
}

pub fn is_legacy_context_block_start(lines: &[&str], index: usize) -> bool {
    lines.get(index).is_some_and(|line| line.trim().is_empty())
        && lines
            .get(index + 1)
            .is_some_and(|line| is_command_context_line(line))
        && lines
            .get(index + 2)
            .is_some_and(|next| next.starts_with("$ "))
}

fn draw_divider(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, color));
}

/// Render the scrollable transcript area for the active terminal session.
pub fn render(ui: &mut egui::Ui, state: &AppState) -> bool {
    let session_idx = state.selected_terminal_idx;
    let lines = state.recent_terminal_lines(session_idx, 64);

    let transcript_ends_with_error = lines
        .iter()
        .rposition(|line| line.starts_with("$ "))
        .is_some_and(|index| command_block_has_error(&lines, index));

    let transcript_height = (ui.available_height() - 110.0).max(180.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(transcript_height)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            let num_lines = lines.len() as f32;
            let num_prompts = lines.iter().filter(|l| l.starts_with("$ ")).count() as f32;
            let estimated_height = (num_lines * 14.5) + (num_prompts * 18.0) + 10.0;
            let remaining_space = ui.available_height() - estimated_height;
            if remaining_space > 0.0 {
                ui.add_space(remaining_space);
            }

            let mut index = 0usize;
            let mut previous_block_had_error = false;
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
                    if should_render_block_separator(previous_block_had_error, has_error) {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                    }

                    let block_width = ui.available_width();
                    let extend_to_bottom = should_extend_error_block_to_bottom(
                        has_error,
                        block_end == lines.len(),
                    );
                    let block_height = if extend_to_bottom {
                        ui.available_height()
                    } else {
                        0.0
                    };
                    Frame::new()
                        .fill(if has_error {
                            ERROR_PANEL_BG
                        } else {
                            Color32::TRANSPARENT
                        })
                        .corner_radius(if has_error { 0.0 } else { 8.0 })
                        .inner_margin(egui::Margin::symmetric(
                            10,
                            if has_error { 16 } else { 8 },
                        ))
                        .show(ui, |ui| {
                            ui.set_min_width((block_width - 20.0).max(0.0));
                            if extend_to_bottom {
                                ui.set_min_height(block_height);
                            }
                            for block_line in &lines[block_start..block_end] {
                                let mut color = FG_PRIMARY;
                                let mut is_bold = false;

                                if is_command_context_line(block_line) {
                                    color = FG_MUTED;
                                } else if block_line.starts_with("$ ") {
                                    color = if has_error { ERROR_TEXT } else { Color32::WHITE };
                                    is_bold = true;
                                } else if block_line.starts_with("~ (") {
                                    color = FG_MUTED;
                                } else if is_error_output_line(block_line) {
                                    color = ERROR_TEXT;
                                }

                                ui.label(if is_bold {
                                    RichText::new(*block_line).monospace().color(color).strong()
                                } else {
                                    RichText::new(*block_line).monospace().color(color)
                                });
                            }
                        });

                    previous_block_had_error = has_error;
                    index = block_end;
                    while index < lines.len() && lines[index].trim().is_empty() {
                        index += 1;
                    }
                    continue;
                }

                let color = if line.starts_with("~ (") {
                    FG_MUTED
                } else if is_error_output_line(line) {
                    Color32::from_rgb(255, 205, 205)
                } else {
                    FG_PRIMARY
                };

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(14.0);
                    ui.label(RichText::new(line).monospace().color(color));
                });
                previous_block_had_error = false;
                index += 1;
            }
        });

    if transcript_ends_with_error {
        ui.add_space(0.0);
    } else {
        ui.add_space(16.0);
        draw_divider(ui, Color32::from_gray(60));
        ui.add_space(12.0);
    }

    transcript_ends_with_error
}
