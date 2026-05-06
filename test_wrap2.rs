use eframe::egui::{self, text::LayoutJob, TextFormat, Color32, FontId, FontFamily};
fn main() {
    let mut job = LayoutJob::default();
    job.append("CONTRIBUTING.md       KAKU-learning.md        VIEW-architecture.md ci-cd-architecture.md docs-architecture.md  target", 0.0, TextFormat {
        font_id: FontId::new(14.0, FontFamily::Monospace),
        color: Color32::WHITE,
        ..Default::default()
    });
    job.wrap.max_width = 800.0;
    job.wrap.break_anywhere = true;
    println!("Job created.");
}
