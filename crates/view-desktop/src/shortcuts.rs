//! Keyboard shortcut handling for VIEW Desktop.
//!
//! All application-wide key bindings live here. Surface-specific key handling
//! (e.g., history navigation inside the shell input) remains in their respective
//! render modules.

use eframe::egui::{self, Key};
use view_core::app::AppState;

/// Handle all application-level keyboard shortcuts.
///
/// Called once per frame before any panel rendering. Returns `true` if a
/// shortcut was consumed so callers can short-circuit further input processing.
pub fn handle(ctx: &egui::Context, state: &mut AppState) -> bool {
    let mut consumed = false;

    // ── Agent navigation ───────────────────────────────────────────────────
    if ctx.input(|i| i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J)) {
        state.select_next();
        consumed = true;
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K)) {
        state.select_previous();
        consumed = true;
    }
    if ctx.input(|i| i.key_pressed(Key::PageDown)) {
        state.select_next_page();
        consumed = true;
    }
    if ctx.input(|i| i.key_pressed(Key::PageUp)) {
        state.select_previous_page();
        consumed = true;
    }

    // ── Agent filter ───────────────────────────────────────────────────────
    if ctx.input(|i| i.key_pressed(Key::F) && !i.modifiers.ctrl && !i.modifiers.command) {
        state.cycle_filter_mode();
        consumed = true;
    }

    // ── Terminal tab management (Cmd+T / Cmd+W / Cmd+1..9) ────────────────
    if ctx.input(|i| i.key_pressed(Key::T) && i.modifiers.command) {
        // New terminal — caller must wire a new shell process via add_terminal_session
        let count = state.terminal_sessions().len();
        state.add_terminal_session(format!("shell-{}", count + 1));
        consumed = true;
    }
    if ctx.input(|i| i.key_pressed(Key::W) && i.modifiers.command) {
        let idx = state.selected_terminal_idx;
        state.remove_terminal_session(idx);
        consumed = true;
    }

    // Cmd+1 … Cmd+9 — switch terminal by index
    let digit_keys = [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ];
    for (tab_index, &key) in digit_keys.iter().enumerate() {
        if ctx.input(|i| i.key_pressed(key) && i.modifiers.command) {
            state.select_terminal_index(tab_index);
            consumed = true;
        }
    }

    // Cmd+Shift+] / Cmd+Shift+[ — next/prev terminal tab
    if ctx.input(|i| i.key_pressed(Key::ArrowRight) && i.modifiers.command) {
        let next = state
            .selected_terminal_idx
            .saturating_add(1)
            .min(state.terminal_sessions().len().saturating_sub(1));
        state.select_terminal_index(next);
        consumed = true;
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowLeft) && i.modifiers.command) {
        let prev = state.selected_terminal_idx.saturating_sub(1);
        state.select_terminal_index(prev);
        consumed = true;
    }

    // ── View toggle ────────────────────────────────────────────────────────
    if ctx.input(|i| i.key_pressed(Key::G) && i.modifiers.command) {
        state.toggle_view_mode();
        consumed = true;
    }

    consumed
}
