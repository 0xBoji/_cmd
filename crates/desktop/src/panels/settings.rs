use crate::setup;
use crate::theme::*;
use eframe::egui::{self, Align, Frame, Layout, RichText, Stroke};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Shell,
    Network,
    Shortcuts,
}

pub fn render_shell_settings_panel(
    ui: &mut egui::Ui,
    shell_setup_show_preview: &mut bool,
    shell_setup_status_message: &mut Option<(String, bool)>,
) {
    ui.label(
        RichText::new("Shell")
            .color(SETTINGS_TEXT_PRIMARY)
            .size(20.0)
            .strong(),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new("Manage the generated zsh integration used by _CMD.")
            .color(SETTINGS_TEXT_SECONDARY)
            .size(14.0),
    );
    ui.add_space(18.0);

    match setup::inspect_shell_setup() {
        Ok(status) => {
            Frame::new()
                .fill(SETTINGS_CARD_BG)
                .stroke(Stroke::new(1.0, SETTINGS_BORDER))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::symmetric(18, 16))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Managed zsh integration")
                                .color(SETTINGS_TEXT_PRIMARY)
                                .size(15.0)
                                .strong(),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(if status.zshrc_patched {
                                    "Installed"
                                } else {
                                    "Not installed"
                                })
                                .color(if status.zshrc_patched {
                                    POSITIVE_GREEN
                                } else {
                                    FG_MUTED
                                })
                                .monospace(),
                            );
                        });
                    });
                    ui.add_space(10.0);
                    ui.add(egui::Label::new(
                        RichText::new(format!("Managed file: {}", status.managed_path.display()))
                            .color(SETTINGS_TEXT_SECONDARY)
                            .size(12.5)
                            .monospace()
                    ).truncate());
                    ui.label(
                        RichText::new(format!(
                            "Managed file status: {}",
                            if status.managed_file_exists {
                                "present"
                            } else {
                                "missing"
                            }
                        ))
                        .color(if status.managed_file_exists {
                            POSITIVE_GREEN
                        } else {
                            FG_MUTED
                        })
                        .size(12.5)
                        .monospace(),
                    );
                    ui.add(egui::Label::new(
                        RichText::new(format!("Shell rc: {}", status.zshrc_path.display()))
                            .color(SETTINGS_TEXT_SECONDARY)
                            .size(12.5)
                            .monospace()
                    ).truncate());
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Install").clicked() {
                            match setup::run_setup_shell() {
                                Ok(()) => {
                                    *shell_setup_status_message = Some((
                                        "Managed zsh integration installed.".to_string(),
                                        false,
                                    ));
                                }
                                Err(error) => {
                                    *shell_setup_status_message =
                                        Some((format!("Install failed: {error}"), true));
                                }
                            }
                        }
                        if ui.button("Preview").clicked() {
                            *shell_setup_show_preview = !*shell_setup_show_preview;
                        }
                        if ui.button("Reset").clicked() {
                            match setup::run_reset_shell() {
                                Ok(()) => {
                                    *shell_setup_status_message = Some((
                                        "Managed zsh integration removed.".to_string(),
                                        false,
                                    ));
                                }
                                Err(error) => {
                                    *shell_setup_status_message =
                                        Some((format!("Reset failed: {error}"), true));
                                }
                            }
                        }
                    });
                });
        }
        Err(error) => {
            ui.label(
                RichText::new(format!("Status unavailable: {error}"))
                    .color(ERROR_COMMAND_TEXT)
                    .monospace(),
            );
        }
    }

    if let Some((message, is_error)) = shell_setup_status_message.as_ref() {
        ui.add_space(14.0);
        ui.label(
            RichText::new(message)
                .color(if *is_error {
                    ERROR_COMMAND_TEXT
                } else {
                    POSITIVE_GREEN
                })
                .size(12.5)
                .monospace(),
        );
    }

    if *shell_setup_show_preview {
        ui.add_space(18.0);
        Frame::new()
            .fill(SETTINGS_CARD_BG)
            .stroke(Stroke::new(1.0, SETTINGS_BORDER))
            .corner_radius(12.0)
            .inner_margin(egui::Margin::symmetric(14, 14))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Managed block preview")
                        .color(SETTINGS_TEXT_PRIMARY)
                        .size(14.0)
                        .strong(),
                );
                ui.add_space(8.0);
                let mut preview = setup::preview_shell_init()
                    .unwrap_or_else(|error| format!("Failed to render preview: {error}"));
                ui.add(
                    egui::TextEdit::multiline(&mut preview)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(8)
                        .interactive(false),
                );
            });
    }
}
