use crate::theme::*;
use eframe::egui::{self, Color32, RichText, Stroke};

pub fn titlebar_icon_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    if response.hovered() || active {
        ui.painter().rect_filled(
            rect,
            8.0,
            if active {
                Color32::from_rgb(52, 52, 56)
            } else {
                Color32::from_rgb(38, 38, 42)
            },
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(15.0),
        if active { FG_PRIMARY } else { FG_MUTED },
    );
    response
}

pub fn terminal_directory_button(label: String) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(label)
            .color(FG_PRIMARY)
            .size(11.0)
            .monospace(),
    )
    .stroke(Stroke::new(1.0, Color32::from_gray(60)))
    .corner_radius(5.0)
    .fill(BG_PANEL_ALT)
    .min_size(egui::vec2(0.0, 22.0))
}

pub fn terminal_branch_button(label: String) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(label)
            .color(BRANCH_GREEN)
            .size(11.0)
            .monospace(),
    )
    .stroke(Stroke::new(1.0, BRANCH_GREEN_BORDER))
    .corner_radius(5.0)
    .fill(BG_PANEL)
    .min_size(egui::vec2(0.0, 20.0))
}

pub fn draw_branch_icon(painter: &egui::Painter, left: f32, center_y: f32, color: Color32) {
    // Compact version: ~70% of original size
    let stroke = Stroke::new(1.1, color);
    let top    = egui::pos2(left + 2.5, center_y - 3.5);
    let mid    = egui::pos2(left + 2.5, center_y + 0.5);
    let right  = egui::pos2(left + 6.5, center_y - 1.0);
    let bottom = egui::pos2(left + 2.5, center_y + 3.5);

    painter.line_segment([top, mid], stroke);
    painter.line_segment([mid, right], stroke);
    painter.line_segment([mid, bottom], stroke);
    painter.circle_stroke(top,   1.4, stroke);
    painter.circle_stroke(right, 1.4, stroke);
    painter.circle_stroke(bottom, 1.4, stroke);
}

pub fn settings_sidebar_row(
    ui: &mut egui::Ui,
    label: &str,
    icon: &str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(196.0, 38.0), egui::Sense::click());
    if response.hovered() || selected {
        ui.painter().rect_filled(
            rect,
            10.0,
            if selected {
                SETTINGS_SELECTED_ROW_BG
            } else {
                SETTINGS_HOVER_ROW_BG
            },
        );
    }
    ui.painter().text(
        egui::pos2(rect.min.x + 14.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{icon}  {label}"),
        egui::FontId::proportional(13.5),
        if selected {
            SETTINGS_TEXT_PRIMARY
        } else {
            SETTINGS_TEXT_SECONDARY
        },
    );
    response
}

