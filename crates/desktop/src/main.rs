pub mod components;
pub mod panels;
pub mod utils;
mod desktop_app;
mod setup;
mod shell;
mod shortcuts;
pub mod theme;
mod transcript;

use anyhow::Result;
use desktop_app::CmdDesktopApp;
use eframe::{egui, Renderer};
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match setup::parse_command(&args) {
        setup::SetupCommand::LaunchUi => launch_ui(),
        setup::SetupCommand::SetupShell => setup::run_setup_shell(),
        setup::SetupCommand::ResetShell => setup::run_reset_shell(),
        setup::SetupCommand::PrintShellInit => setup::print_shell_init(),
    }
}

fn launch_ui() -> Result<()> {
    main_debug_log("desktop main: starting".to_string());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("_CMD Desktop")
            .with_inner_size([1600.0, 980.0])
            .with_min_inner_size([960.0, 640.0])
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_transparent(true),
        renderer: Renderer::Glow,
        ..eframe::NativeOptions::default()
    };

    main_debug_log("desktop main: calling run_native".to_string());
    let result = eframe::run_native(
        "_CMD Desktop",
        options,
        Box::new(|cc| {
            main_debug_log("desktop main: app creator invoked".to_string());
            Ok(Box::new(CmdDesktopApp::new(cc)))
        }),
    );
    main_debug_log(format!(
        "desktop main: run_native returned {:?}",
        result.as_ref().err()
    ));
    result.map_err(|error| anyhow::anyhow!("Failed to launch _CMD Desktop: {error}"))
}

fn main_debug_log(message: String) {
    let Ok(path) = std::env::var("VIEW_DESKTOP_DEBUG_LOG") else {
        return;
    };

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}
