use crate::theme::*;
use eframe::egui::{self, Color32, Frame, RichText, Stroke};

pub fn show_button_tooltip(ctx: &egui::Context, id: &'static str, rect: egui::Rect, text: &str) {
    let pos = egui::pos2(rect.min.x, rect.min.y - 40.0);
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            Frame::new()
                .fill(Color32::from_gray(185))
                .stroke(Stroke::new(1.0, Color32::from_gray(195)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).color(BG_APP).size(11.5));
                });
        });
}
