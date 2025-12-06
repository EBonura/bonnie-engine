//! Tracker UI layout and rendering

use macroquad::prelude::*;
use crate::ui::{Rect, UiContext};
use super::state::{TrackerState, TrackerView};
use super::pattern::NUM_CHANNELS;

// Colors
const BG_COLOR: Color = Color::new(0.11, 0.11, 0.13, 1.0);
const HEADER_COLOR: Color = Color::new(0.15, 0.15, 0.18, 1.0);
const ROW_EVEN: Color = Color::new(0.13, 0.13, 0.15, 1.0);
const ROW_ODD: Color = Color::new(0.11, 0.11, 0.13, 1.0);
const ROW_BEAT: Color = Color::new(0.16, 0.14, 0.12, 1.0);
const ROW_HIGHLIGHT: Color = Color::new(0.2, 0.25, 0.3, 1.0);
const CURSOR_COLOR: Color = Color::new(0.3, 0.5, 0.8, 0.8);
const PLAYBACK_ROW_COLOR: Color = Color::new(0.4, 0.2, 0.2, 0.6);
const TEXT_COLOR: Color = Color::new(0.8, 0.8, 0.85, 1.0);
const TEXT_DIM: Color = Color::new(0.4, 0.4, 0.45, 1.0);
const NOTE_COLOR: Color = Color::new(0.9, 0.85, 0.5, 1.0);
const INST_COLOR: Color = Color::new(0.5, 0.8, 0.5, 1.0);
const VOL_COLOR: Color = Color::new(0.5, 0.7, 0.9, 1.0);
const FX_COLOR: Color = Color::new(0.9, 0.5, 0.7, 1.0);

// Layout constants
const ROW_HEIGHT: f32 = 18.0;
const CHANNEL_WIDTH: f32 = 140.0;
const ROW_NUM_WIDTH: f32 = 30.0;
const NOTE_WIDTH: f32 = 36.0;
const INST_WIDTH: f32 = 24.0;
const VOL_WIDTH: f32 = 24.0;
const FX_WIDTH: f32 = 16.0;
const FXPARAM_WIDTH: f32 = 24.0;

/// Draw the tracker interface
pub fn draw_tracker(ctx: &mut UiContext, rect: Rect, state: &mut TrackerState) {
    // Background
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BG_COLOR);

    // Split into header and main area
    let header_height = 60.0;
    let header_rect = Rect::new(rect.x, rect.y, rect.w, header_height);
    let main_rect = Rect::new(rect.x, rect.y + header_height, rect.w, rect.h - header_height);

    // Draw header (transport, info)
    draw_header(ctx, header_rect, state);

    // Draw main content based on view
    match state.view {
        TrackerView::Pattern => draw_pattern_view(ctx, main_rect, state),
        TrackerView::Arrangement => draw_arrangement_view(ctx, main_rect, state),
        TrackerView::Instruments => draw_instruments_view(ctx, main_rect, state),
    }

    // Handle input
    handle_input(ctx, state);
}

/// Helper to draw a clickable button, returns true if clicked
fn draw_button(ctx: &UiContext, x: f32, y: f32, w: f32, h: f32, label: &str, bg_color: Color) -> bool {
    let rect = Rect::new(x, y, w, h);
    let hovered = ctx.mouse.inside(&rect);
    let color = if hovered {
        Color::new(bg_color.r + 0.1, bg_color.g + 0.1, bg_color.b + 0.1, bg_color.a)
    } else {
        bg_color
    };

    draw_rectangle(x, y, w, h, color);
    draw_text(label, x + 4.0, y + h - 6.0, 14.0, TEXT_COLOR);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Helper to draw a value control with +/- buttons, returns delta (-1, 0, or 1)
fn draw_value_control(ctx: &UiContext, x: f32, y: f32, label: &str, value: &str, color: Color) -> i32 {
    let btn_size = 18.0;
    let label_w = 45.0;
    let value_w = 30.0;

    // Label
    draw_text(label, x, y + 14.0, 12.0, TEXT_DIM);

    // Value
    draw_text(value, x + label_w, y + 14.0, 14.0, color);

    // - button
    let minus_rect = Rect::new(x + label_w + value_w, y + 1.0, btn_size, btn_size);
    let minus_hovered = ctx.mouse.inside(&minus_rect);
    draw_rectangle(minus_rect.x, minus_rect.y, minus_rect.w, minus_rect.h,
        if minus_hovered { Color::new(0.35, 0.25, 0.25, 1.0) } else { Color::new(0.25, 0.2, 0.2, 1.0) });
    draw_text("-", minus_rect.x + 5.0, minus_rect.y + 13.0, 14.0, TEXT_COLOR);

    // + button
    let plus_rect = Rect::new(x + label_w + value_w + btn_size + 2.0, y + 1.0, btn_size, btn_size);
    let plus_hovered = ctx.mouse.inside(&plus_rect);
    draw_rectangle(plus_rect.x, plus_rect.y, plus_rect.w, plus_rect.h,
        if plus_hovered { Color::new(0.25, 0.35, 0.25, 1.0) } else { Color::new(0.2, 0.25, 0.2, 1.0) });
    draw_text("+", plus_rect.x + 4.0, plus_rect.y + 13.0, 14.0, TEXT_COLOR);

    if minus_hovered && is_mouse_button_pressed(MouseButton::Left) {
        -1
    } else if plus_hovered && is_mouse_button_pressed(MouseButton::Left) {
        1
    } else {
        0
    }
}

/// Draw the header with transport controls and song info
fn draw_header(ctx: &mut UiContext, rect: Rect, state: &mut TrackerState) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, HEADER_COLOR);

    let mut x = rect.x + 10.0;
    let y = rect.y + 8.0;

    // View mode buttons
    let views = [
        (TrackerView::Pattern, "Pattern"),
        (TrackerView::Arrangement, "Arrange"),
        (TrackerView::Instruments, "Instr"),
    ];

    for (view, label) in views {
        let btn_w = 65.0;
        let is_active = state.view == view;
        let color = if is_active {
            Color::new(0.3, 0.4, 0.5, 1.0)
        } else {
            Color::new(0.2, 0.2, 0.25, 1.0)
        };

        if draw_button(ctx, x, y, btn_w, 20.0, label, color) {
            state.view = view;
        }

        x += btn_w + 4.0;
    }

    x += 15.0;

    // Transport controls - multiple buttons
    // Stop button (rewind to start)
    if draw_button(ctx, x, y, 35.0, 20.0, "<<", Color::new(0.25, 0.2, 0.2, 1.0)) {
        state.stop_playback();
    }
    x += 37.0;

    // Play from start button
    if draw_button(ctx, x, y, 35.0, 20.0, "|>", Color::new(0.2, 0.3, 0.2, 1.0)) {
        state.play_from_start();
    }
    x += 37.0;

    // Play/pause from cursor
    let play_label = if state.playing { "||" } else { ">" };
    if draw_button(ctx, x, y, 35.0, 20.0, play_label, Color::new(0.2, 0.25, 0.3, 1.0)) {
        state.toggle_playback();
    }
    x += 50.0;

    // Value controls
    let delta = draw_value_control(ctx, x, y, "BPM:", &format!("{:3}", state.song.bpm), TEXT_COLOR);
    if delta != 0 {
        state.song.bpm = (state.song.bpm as i32 + delta * 5).clamp(40, 300) as u16;
    }
    x += 135.0;

    let delta = draw_value_control(ctx, x, y, "Oct:", &format!("{}", state.octave), NOTE_COLOR);
    if delta != 0 {
        state.octave = (state.octave as i32 + delta).clamp(0, 9) as u8;
    }
    x += 115.0;

    let delta = draw_value_control(ctx, x, y, "Step:", &format!("{}", state.edit_step), TEXT_COLOR);
    if delta != 0 {
        state.edit_step = (state.edit_step as i32 + delta).clamp(0, 16) as usize;
    }
    x += 120.0;

    let delta = draw_value_control(ctx, x, y, "Inst:", &format!("{:02}", state.current_instrument), INST_COLOR);
    if delta != 0 {
        state.current_instrument = (state.current_instrument as i32 + delta).clamp(0, 127) as u8;
        state.audio.set_program(state.current_channel as i32, state.current_instrument as i32);
    }

    // Second row - position info and soundfont status
    let y2 = y + 26.0;
    let pattern_num = state.song.arrangement.get(state.current_pattern_idx).copied().unwrap_or(0);
    draw_text(
        &format!("Pos: {:02}/{:02}  Pat: {:02}  Row: {:03}/{:03}  Ch: {}",
                 state.current_pattern_idx,
                 state.song.arrangement.len(),
                 pattern_num,
                 state.current_row,
                 state.current_pattern().map(|p| p.length).unwrap_or(64),
                 state.current_channel + 1),
        rect.x + 10.0, y2 + 14.0, 12.0, TEXT_COLOR
    );

    // Soundfont status
    let sf_status = state.audio.soundfont_name()
        .map(|n| format!("SF: {}", n))
        .unwrap_or_else(|| "No Soundfont".to_string());
    draw_text(&sf_status, rect.x + 350.0, y2 + 14.0, 12.0, if state.audio.is_loaded() { TEXT_DIM } else { Color::new(0.8, 0.3, 0.3, 1.0) });

    // Status message
    if let Some(status) = state.get_status() {
        draw_text(status, rect.x + 550.0, y2 + 14.0, 12.0, Color::new(1.0, 0.8, 0.3, 1.0));
    }
}

/// Draw the pattern editor view
fn draw_pattern_view(ctx: &mut UiContext, rect: Rect, state: &mut TrackerState) {
    // Calculate visible rows first (before borrowing pattern)
    state.visible_rows = ((rect.h - ROW_HEIGHT) / ROW_HEIGHT) as usize;

    // Get pattern info without holding borrow
    let (pattern_length, rows_per_beat) = match state.current_pattern() {
        Some(p) => (p.length, state.song.rows_per_beat),
        None => return,
    };

    // Channel header
    draw_rectangle(rect.x, rect.y, rect.w, ROW_HEIGHT, HEADER_COLOR);

    let mut x = rect.x + ROW_NUM_WIDTH;
    for ch in 0..NUM_CHANNELS {
        let ch_x = x;
        let header_rect = Rect::new(ch_x, rect.y, CHANNEL_WIDTH, ROW_HEIGHT);

        // Highlight on hover
        if ctx.mouse.inside(&header_rect) {
            draw_rectangle(ch_x, rect.y, CHANNEL_WIDTH, ROW_HEIGHT, Color::new(0.25, 0.25, 0.3, 1.0));

            // Click to select channel
            if is_mouse_button_pressed(MouseButton::Left) {
                state.current_channel = ch;
            }
        }

        draw_text(&format!("Ch {}", ch + 1), ch_x + 4.0, rect.y + 14.0, 12.0, TEXT_COLOR);
        x += CHANNEL_WIDTH;

        // Channel separator
        draw_line(x - 1.0, rect.y, x - 1.0, rect.y + rect.h, 1.0, Color::new(0.25, 0.25, 0.3, 1.0));
    }

    // Handle mouse clicks on pattern grid
    let grid_y_start = rect.y + ROW_HEIGHT;
    let grid_rect = Rect::new(rect.x, grid_y_start, rect.w, rect.h - ROW_HEIGHT);

    if ctx.mouse.inside(&grid_rect) && is_mouse_button_pressed(MouseButton::Left) {
        let mouse_x = ctx.mouse.x;
        let mouse_y = ctx.mouse.y;

        // Calculate clicked row
        let clicked_screen_row = ((mouse_y - grid_y_start) / ROW_HEIGHT) as usize;
        let clicked_row = state.scroll_row + clicked_screen_row;

        if clicked_row < pattern_length {
            state.current_row = clicked_row;

            // Calculate clicked channel and column
            let rel_x = mouse_x - rect.x - ROW_NUM_WIDTH;
            if rel_x >= 0.0 {
                let clicked_channel = (rel_x / CHANNEL_WIDTH) as usize;
                if clicked_channel < NUM_CHANNELS {
                    state.current_channel = clicked_channel;

                    // Calculate column within channel
                    let col_x = rel_x - (clicked_channel as f32 * CHANNEL_WIDTH);
                    state.current_column = if col_x < NOTE_WIDTH {
                        0 // Note
                    } else if col_x < NOTE_WIDTH + INST_WIDTH {
                        1 // Instrument
                    } else if col_x < NOTE_WIDTH + INST_WIDTH + VOL_WIDTH {
                        2 // Volume
                    } else if col_x < NOTE_WIDTH + INST_WIDTH + VOL_WIDTH + FX_WIDTH {
                        3 // Effect
                    } else {
                        4 // Effect param
                    };
                }
            }
        }
    }

    // Now re-borrow pattern for drawing
    let pattern = match state.current_pattern() {
        Some(p) => p,
        None => return,
    };

    // Draw rows
    let start_row = state.scroll_row;
    let visible_rows = state.visible_rows;
    let end_row = (start_row + visible_rows).min(pattern.length);

    for row_idx in start_row..end_row {
        let screen_row = row_idx - start_row;
        let y = rect.y + ROW_HEIGHT + screen_row as f32 * ROW_HEIGHT;

        // Row background
        let row_bg = if state.playing && row_idx == state.playback_row && state.playback_pattern_idx == state.current_pattern_idx {
            PLAYBACK_ROW_COLOR
        } else if row_idx == state.current_row {
            ROW_HIGHLIGHT
        } else if row_idx % (rows_per_beat as usize * 4) == 0 {
            ROW_BEAT
        } else if row_idx % 2 == 0 {
            ROW_EVEN
        } else {
            ROW_ODD
        };
        draw_rectangle(rect.x, y, rect.w, ROW_HEIGHT, row_bg);

        // Row number
        let row_color = if row_idx % (rows_per_beat as usize) == 0 { TEXT_COLOR } else { TEXT_DIM };
        draw_text(&format!("{:02X}", row_idx), rect.x + 4.0, y + 14.0, 12.0, row_color);

        // Draw each channel
        let mut x = rect.x + ROW_NUM_WIDTH;
        for ch in 0..NUM_CHANNELS {
            let note = &pattern.channels[ch][row_idx];

            // Cursor highlight
            if row_idx == state.current_row && ch == state.current_channel {
                let col_x = x + match state.current_column {
                    0 => 0.0,
                    1 => NOTE_WIDTH,
                    2 => NOTE_WIDTH + INST_WIDTH,
                    3 => NOTE_WIDTH + INST_WIDTH + VOL_WIDTH,
                    _ => NOTE_WIDTH + INST_WIDTH + VOL_WIDTH + FX_WIDTH,
                };
                let col_w = match state.current_column {
                    0 => NOTE_WIDTH,
                    1 => INST_WIDTH,
                    2 => VOL_WIDTH,
                    3 => FX_WIDTH,
                    _ => FXPARAM_WIDTH,
                };
                draw_rectangle(col_x, y, col_w, ROW_HEIGHT, CURSOR_COLOR);
            }

            // Note
            let note_str = note.pitch_name().unwrap_or_else(|| "---".to_string());
            let note_color = if note.pitch.is_some() { NOTE_COLOR } else { TEXT_DIM };
            draw_text(&note_str, x + 2.0, y + 14.0, 12.0, note_color);

            // Instrument
            let inst_str = note.instrument.map(|i| format!("{:02X}", i)).unwrap_or_else(|| "--".to_string());
            let inst_color = if note.instrument.is_some() { INST_COLOR } else { TEXT_DIM };
            draw_text(&inst_str, x + NOTE_WIDTH + 2.0, y + 14.0, 12.0, inst_color);

            // Volume
            let vol_str = note.volume.map(|v| format!("{:02X}", v)).unwrap_or_else(|| "--".to_string());
            let vol_color = if note.volume.is_some() { VOL_COLOR } else { TEXT_DIM };
            draw_text(&vol_str, x + NOTE_WIDTH + INST_WIDTH + 2.0, y + 14.0, 12.0, vol_color);

            // Effect
            let fx_str = note.effect.map(|e| e.to_string()).unwrap_or_else(|| "-".to_string());
            let fx_color = if note.effect.is_some() { FX_COLOR } else { TEXT_DIM };
            draw_text(&fx_str, x + NOTE_WIDTH + INST_WIDTH + VOL_WIDTH + 2.0, y + 14.0, 12.0, fx_color);

            // Effect param
            let fxp_str = note.effect_param.map(|p| format!("{:02X}", p)).unwrap_or_else(|| "--".to_string());
            draw_text(&fxp_str, x + NOTE_WIDTH + INST_WIDTH + VOL_WIDTH + FX_WIDTH + 2.0, y + 14.0, 12.0, fx_color);

            x += CHANNEL_WIDTH;
        }
    }
}

/// Draw the arrangement view (placeholder)
fn draw_arrangement_view(_ctx: &mut UiContext, rect: Rect, state: &TrackerState) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BG_COLOR);

    // Header
    draw_text("Song Arrangement", rect.x + 10.0, rect.y + 24.0, 16.0, TEXT_COLOR);

    // Draw arrangement as list
    let mut y = rect.y + 50.0;
    for (i, &pattern_idx) in state.song.arrangement.iter().enumerate() {
        let is_current = i == state.current_pattern_idx;
        let bg = if is_current { ROW_HIGHLIGHT } else if i % 2 == 0 { ROW_EVEN } else { ROW_ODD };
        draw_rectangle(rect.x + 10.0, y, 200.0, 24.0, bg);
        draw_text(
            &format!("{:02}: Pattern {:02}", i, pattern_idx),
            rect.x + 20.0, y + 16.0, 14.0,
            if is_current { NOTE_COLOR } else { TEXT_COLOR }
        );
        y += 26.0;
    }

    draw_text("(Press + to add pattern, - to remove)", rect.x + 10.0, rect.y + rect.h - 30.0, 12.0, TEXT_DIM);
}

/// Piano key layout for drawing
const PIANO_WHITE_KEYS: [(u8, &str); 7] = [
    (0, "C"), (2, "D"), (4, "E"), (5, "F"), (7, "G"), (9, "A"), (11, "B")
];
const PIANO_BLACK_KEYS: [(u8, &str, f32); 5] = [
    (1, "C#", 0.7), (3, "D#", 1.7), (6, "F#", 3.7), (8, "G#", 4.7), (10, "A#", 5.7)
];

/// Keyboard mapping for piano: maps key offset (0-23) to keyboard key name
fn get_key_label(offset: u8) -> Option<&'static str> {
    match offset {
        0 => Some("Z"), 1 => Some("S"), 2 => Some("X"), 3 => Some("D"), 4 => Some("C"),
        5 => Some("V"), 6 => Some("G"), 7 => Some("B"), 8 => Some("H"), 9 => Some("N"),
        10 => Some("J"), 11 => Some("M"),
        12 => Some("Q"), 13 => Some("2"), 14 => Some("W"), 15 => Some("3"), 16 => Some("E"),
        17 => Some("R"), 18 => Some("5"), 19 => Some("T"), 20 => Some("6"), 21 => Some("Y"),
        22 => Some("7"), 23 => Some("U"),
        _ => None,
    }
}

/// Draw the instruments view with piano keyboard
fn draw_instruments_view(ctx: &mut UiContext, rect: Rect, state: &mut TrackerState) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BG_COLOR);

    // Split into left (instrument list) and right (piano + info)
    let list_width = 280.0;
    let list_rect = Rect::new(rect.x, rect.y, list_width, rect.h);

    // === LEFT: Instrument List ===
    draw_rectangle(list_rect.x, list_rect.y, list_rect.w, list_rect.h, Color::new(0.09, 0.09, 0.11, 1.0));
    draw_text("Instruments (GM)", list_rect.x + 10.0, list_rect.y + 20.0, 14.0, TEXT_COLOR);

    // Scrollable instrument list
    let presets = state.audio.get_preset_names();
    let item_height = 18.0;
    let list_start_y = list_rect.y + 35.0;
    let visible_items = ((list_rect.h - 45.0) / item_height) as usize;

    // Simple scroll based on current instrument
    let scroll_offset = if state.current_instrument as usize > visible_items / 2 {
        (state.current_instrument as usize - visible_items / 2).min(presets.len().saturating_sub(visible_items))
    } else {
        0
    };

    for (i, (_, program, name)) in presets.iter().enumerate().skip(scroll_offset).take(visible_items) {
        let y = list_start_y + (i - scroll_offset) as f32 * item_height;
        let item_rect = Rect::new(list_rect.x + 5.0, y, list_rect.w - 10.0, item_height);

        let is_current = *program == state.current_instrument;
        let is_hovered = ctx.mouse.inside(&item_rect);

        // Background
        let bg = if is_current {
            Color::new(0.25, 0.3, 0.35, 1.0)
        } else if is_hovered {
            Color::new(0.18, 0.18, 0.22, 1.0)
        } else if i % 2 == 0 {
            Color::new(0.11, 0.11, 0.13, 1.0)
        } else {
            Color::new(0.09, 0.09, 0.11, 1.0)
        };
        draw_rectangle(item_rect.x, item_rect.y, item_rect.w, item_rect.h, bg);

        // Click to select
        if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
            state.current_instrument = *program;
            state.audio.set_program(state.current_channel as i32, *program as i32);
        }

        // Text
        let color = if is_current { NOTE_COLOR } else { TEXT_COLOR };
        draw_text(&format!("{:03}: {}", program, name), item_rect.x + 5.0, y + 13.0, 12.0, color);
    }

    // === RIGHT: Piano Keyboard ===
    let piano_x = rect.x + list_width + 20.0;
    let piano_y = rect.y + 30.0;
    let white_key_w = 36.0;
    let white_key_h = 120.0;
    let black_key_w = 24.0;
    let black_key_h = 75.0;

    draw_text(&format!("Piano - Octave {} & {}", state.octave, state.octave + 1), piano_x, piano_y - 10.0, 14.0, TEXT_COLOR);

    // Draw two octaves of keys
    for octave_offset in 0..2 {
        let octave_x = piano_x + octave_offset as f32 * (7.0 * white_key_w);

        // White keys first (so black keys draw on top)
        for (i, (semitone, note_name)) in PIANO_WHITE_KEYS.iter().enumerate() {
            let key_x = octave_x + i as f32 * white_key_w;
            let key_rect = Rect::new(key_x, piano_y, white_key_w - 2.0, white_key_h);

            let note_offset = octave_offset * 12 + *semitone;
            let midi_note = state.octave * 12 + note_offset;
            let is_hovered = ctx.mouse.inside(&key_rect);

            // Background
            let bg = if is_hovered {
                Color::new(0.85, 0.85, 0.9, 1.0)
            } else {
                Color::new(0.95, 0.95, 0.95, 1.0)
            };
            draw_rectangle(key_x, piano_y, white_key_w - 2.0, white_key_h, bg);
            draw_rectangle(key_x, piano_y, white_key_w - 2.0, white_key_h, Color::new(0.3, 0.3, 0.3, 1.0));
            draw_rectangle(key_x + 1.0, piano_y + 1.0, white_key_w - 4.0, white_key_h - 2.0, bg);

            // Click to play
            if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                state.audio.note_on(state.current_channel as i32, midi_note as i32, 100);
            }
            if is_hovered && is_mouse_button_released(MouseButton::Left) {
                state.audio.note_off(state.current_channel as i32, midi_note as i32);
            }

            // Note name at bottom
            draw_text(note_name, key_x + 12.0, piano_y + white_key_h - 25.0, 14.0, Color::new(0.3, 0.3, 0.3, 1.0));

            // Keyboard shortcut label
            if let Some(key_label) = get_key_label(note_offset) {
                draw_text(key_label, key_x + 13.0, piano_y + white_key_h - 8.0, 12.0, Color::new(0.5, 0.5, 0.5, 1.0));
            }
        }

        // Black keys on top
        for (semitone, _note_name, x_pos) in PIANO_BLACK_KEYS.iter() {
            let key_x = octave_x + *x_pos * white_key_w;
            let key_rect = Rect::new(key_x, piano_y, black_key_w, black_key_h);

            let note_offset = octave_offset * 12 + *semitone;
            let midi_note = state.octave * 12 + note_offset;
            let is_hovered = ctx.mouse.inside(&key_rect);

            // Background
            let bg = if is_hovered {
                Color::new(0.35, 0.35, 0.4, 1.0)
            } else {
                Color::new(0.15, 0.15, 0.18, 1.0)
            };
            draw_rectangle(key_x, piano_y, black_key_w, black_key_h, bg);

            // Click to play
            if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                state.audio.note_on(state.current_channel as i32, midi_note as i32, 100);
            }
            if is_hovered && is_mouse_button_released(MouseButton::Left) {
                state.audio.note_off(state.current_channel as i32, midi_note as i32);
            }

            // Keyboard shortcut label
            if let Some(key_label) = get_key_label(note_offset) {
                draw_text(key_label, key_x + 7.0, piano_y + black_key_h - 8.0, 10.0, Color::new(0.6, 0.6, 0.6, 1.0));
            }
        }
    }

    // Current instrument info below piano
    let info_y = piano_y + white_key_h + 30.0;
    let current_name = presets.iter()
        .find(|(_, p, _)| *p == state.current_instrument)
        .map(|(_, _, n)| n.as_str())
        .unwrap_or("Unknown");

    draw_text(&format!("Current: {:03} - {}", state.current_instrument, current_name),
              piano_x, info_y, 16.0, INST_COLOR);

    // Help text
    draw_text("Click keys to preview | Use keyboard (Z-M, Q-U) to enter notes",
              piano_x, info_y + 25.0, 12.0, TEXT_DIM);
    draw_text("[ ] = prev/next instrument | +/- = octave up/down",
              piano_x, info_y + 42.0, 12.0, TEXT_DIM);
}

/// Handle keyboard and mouse input
fn handle_input(_ctx: &mut UiContext, state: &mut TrackerState) {
    // Navigation
    if is_key_pressed(KeyCode::Up) {
        state.cursor_up();
    }
    if is_key_pressed(KeyCode::Down) {
        state.cursor_down();
    }
    if is_key_pressed(KeyCode::Left) {
        state.cursor_left();
    }
    if is_key_pressed(KeyCode::Right) {
        state.cursor_right();
    }
    if is_key_pressed(KeyCode::Tab) {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            state.prev_channel();
        } else {
            state.next_channel();
        }
    }

    // Page up/down
    if is_key_pressed(KeyCode::PageUp) {
        for _ in 0..16 {
            state.cursor_up();
        }
    }
    if is_key_pressed(KeyCode::PageDown) {
        for _ in 0..16 {
            state.cursor_down();
        }
    }

    // Home/End
    if is_key_pressed(KeyCode::Home) {
        state.current_row = 0;
        state.scroll_row = 0;
    }
    if is_key_pressed(KeyCode::End) {
        if let Some(pattern) = state.current_pattern() {
            state.current_row = pattern.length - 1;
        }
    }

    // Playback
    if is_key_pressed(KeyCode::Space) {
        state.toggle_playback();
    }
    if is_key_pressed(KeyCode::Escape) {
        state.stop_playback();
    }

    // Octave
    if is_key_pressed(KeyCode::KpAdd) || (is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::Equal)) {
        state.octave = (state.octave + 1).min(9);
        state.set_status(&format!("Octave: {}", state.octave), 1.0);
    }
    if is_key_pressed(KeyCode::KpSubtract) || is_key_pressed(KeyCode::Minus) {
        state.octave = state.octave.saturating_sub(1);
        state.set_status(&format!("Octave: {}", state.octave), 1.0);
    }

    // Instrument selection
    if is_key_pressed(KeyCode::LeftBracket) {
        state.current_instrument = state.current_instrument.saturating_sub(1);
        state.audio.set_program(state.current_channel as i32, state.current_instrument as i32);
        state.set_status(&format!("Instrument: {:02}", state.current_instrument), 1.0);
    }
    if is_key_pressed(KeyCode::RightBracket) {
        state.current_instrument = (state.current_instrument + 1).min(127);
        state.audio.set_program(state.current_channel as i32, state.current_instrument as i32);
        state.set_status(&format!("Instrument: {:02}", state.current_instrument), 1.0);
    }

    // Edit step
    if is_key_pressed(KeyCode::F9) {
        state.edit_step = state.edit_step.saturating_sub(1);
        state.set_status(&format!("Edit step: {}", state.edit_step), 1.0);
    }
    if is_key_pressed(KeyCode::F10) {
        state.edit_step = (state.edit_step + 1).min(16);
        state.set_status(&format!("Edit step: {}", state.edit_step), 1.0);
    }

    // Delete
    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
        state.delete_note();
    }

    // Note entry (when in edit mode and in note column)
    if state.edit_mode && state.current_column == 0 {
        // Check for note keys
        let note_keys = [
            KeyCode::Z, KeyCode::S, KeyCode::X, KeyCode::D, KeyCode::C,
            KeyCode::V, KeyCode::G, KeyCode::B, KeyCode::H, KeyCode::N,
            KeyCode::J, KeyCode::M,
            KeyCode::Q, KeyCode::Key2, KeyCode::W, KeyCode::Key3, KeyCode::E,
            KeyCode::R, KeyCode::Key5, KeyCode::T, KeyCode::Key6, KeyCode::Y,
            KeyCode::Key7, KeyCode::U,
        ];

        for key in note_keys {
            if is_key_pressed(key) {
                if let Some(pitch) = TrackerState::key_to_note(key, state.octave) {
                    state.enter_note(pitch);
                }
            }
        }

        // Note off with period or backtick
        if is_key_pressed(KeyCode::Period) || is_key_pressed(KeyCode::Apostrophe) {
            state.enter_note_off();
        }
    }
}
