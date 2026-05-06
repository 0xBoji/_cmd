use crate::theme::*;
use eframe::egui::{self, Color32, Stroke};

pub fn draw_divider(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, color),
    );
}
