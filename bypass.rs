pub fn manual_label(ui: &mut egui::Ui, job: egui::text::LayoutJob) -> egui::Response {
    let galley = ui.fonts(|f| f.layout_job(job));
    let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().galley(rect.min, galley, egui::Color32::WHITE);
    }
    response
}
