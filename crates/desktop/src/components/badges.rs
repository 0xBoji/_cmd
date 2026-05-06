use eframe::egui::{self, Color32, Frame, RichText, Stroke};

pub fn render_copied_badge(ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("copied_badge"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-18.0, -18.0])
        .show(ctx, |ui| {
            Frame::new()
                .fill(Color32::from_rgb(146, 102, 245))
                .stroke(Stroke::NONE)
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Copied")
                            .color(Color32::from_rgb(244, 238, 255))
                            .size(13.0)
                            .monospace(),
                    );
                });
        });
}
