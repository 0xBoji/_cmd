use eframe::egui;
use crate::theme::{TERMINAL_FOOTER_RESERVED_HEIGHT, TERMINAL_TRANSCRIPT_MIN_HEIGHT, TERMINAL_INPUT_HEIGHT, TERMINAL_FOOTER_TOP_GAP, TERMINAL_FOOTER_ROW_GAP};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalFooterMetrics {
    pub transcript_height: f32,
    pub input_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalPaneSections {
    pub transcript_rect: egui::Rect,
    pub footer_rect: egui::Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalFooterLayout {
    pub chips_rect: egui::Rect,
    pub input_rect: egui::Rect,
}

pub fn terminal_footer_metrics(available_height: f32) -> TerminalFooterMetrics {
    TerminalFooterMetrics {
        transcript_height: (available_height - TERMINAL_FOOTER_RESERVED_HEIGHT)
            .max(TERMINAL_TRANSCRIPT_MIN_HEIGHT),
        input_height: TERMINAL_INPUT_HEIGHT,
    }
}

pub fn terminal_pane_sections(rect: egui::Rect) -> TerminalPaneSections {
    let footer_height = (rect.height() - terminal_footer_metrics(rect.height()).transcript_height)
        .max(TERMINAL_FOOTER_RESERVED_HEIGHT);
    let footer_min_y = (rect.max.y - footer_height).max(rect.min.y);

    TerminalPaneSections {
        transcript_rect: egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, footer_min_y)),
        footer_rect: egui::Rect::from_min_max(egui::pos2(rect.min.x, footer_min_y), rect.max),
    }
}

pub fn terminal_footer_layout(rect: egui::Rect) -> TerminalFooterLayout {
    let chips_height = 52.0; // 2 rows: dir (22px) + gap (8px) + branch (22px)
    let chips_top = rect.min.y + TERMINAL_FOOTER_TOP_GAP;
    let input_top = chips_top + chips_height + TERMINAL_FOOTER_ROW_GAP;

    TerminalFooterLayout {
        chips_rect: egui::Rect::from_min_size(
            egui::pos2(rect.min.x, chips_top),
            egui::vec2(rect.width(), chips_height),
        ),
        input_rect: egui::Rect::from_min_size(
            egui::pos2(rect.min.x, input_top),
            egui::vec2(rect.width(), TERMINAL_INPUT_HEIGHT),
        ),
    }
}
