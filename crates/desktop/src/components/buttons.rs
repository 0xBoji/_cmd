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
    .stroke(Stroke::new(1.0, Color32::from_gray(80)))
    .corner_radius(5.0)
    .fill(Color32::from_rgb(26, 28, 38))
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

/// Folder icon — mirrors Warp's `folder.svg`: rounded-rect body + tab on top-left.
pub fn draw_folder_icon(painter: &egui::Painter, center: egui::Pos2, size: f32, color: Color32) {
    let half = size * 0.5;
    // Tab: filled rounded rect at top-left corner
    let tab = egui::Rect::from_min_size(
        egui::pos2(center.x - half, center.y - half),
        egui::vec2(size * 0.42, size * 0.26),
    );
    painter.rect_filled(tab, 2.0, color);
    // Body: outlined rounded rect
    let body = egui::Rect::from_center_size(
        egui::pos2(center.x, center.y + size * 0.08),
        egui::vec2(size, size * 0.70),
    );
    painter.rect_stroke(body, 2.0, egui::Stroke::new(1.1, color), egui::StrokeKind::Middle);

}

/// Git-branch icon — mirrors Warp's `git-branch-02.svg`:
/// vertical trunk (root→tip) + one side node branching off upper trunk.
pub fn draw_branch_icon(painter: &egui::Painter, left: f32, center_y: f32, color: Color32) {
    let stroke = Stroke::new(1.1, color);
    let r = 1.4_f32;

    let tip    = egui::pos2(left + 2.5, center_y - 3.5); // top node
    let root   = egui::pos2(left + 2.5, center_y + 3.5); // bottom node
    let side   = egui::pos2(left + 6.5, center_y - 1.0); // branch node

    // Trunk
    painter.line_segment([tip, root], stroke);
    // Arm to side node (from upper trunk)
    painter.line_segment([egui::pos2(left + 2.5, center_y + 0.5), side], stroke);

    painter.circle_stroke(tip,  r, stroke);
    painter.circle_stroke(root, r, stroke);
    painter.circle_stroke(side, r, stroke);
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

