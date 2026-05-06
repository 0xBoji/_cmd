use eframe::egui;
fn main() {
    let job = egui::text::LayoutJob::default();
    println!("{:?}", job.wrap.max_rows);
}
