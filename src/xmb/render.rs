//! XMB Rendering
//!
//! Renders the XMB menu with PS1-style aesthetics using macroquad

use super::state::XMBState;
use macroquad::prelude::*;

/// Font to use for XMB rendering (None = default macroquad font)
pub type XMBFont = Option<Font>;

/// XMB visual theme colors
pub mod theme {
    use macroquad::prelude::Color;

    /// Background gradient top color (dark blue)
    pub const BG_TOP: Color = Color::new(0.04, 0.04, 0.18, 1.0);
    /// Background gradient bottom color (black)
    pub const BG_BOTTOM: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    /// Selected item color (cyan)
    pub const SELECTED: Color = Color::new(0.0, 0.83, 1.0, 1.0);
    /// Unselected item color (gray)
    pub const UNSELECTED: Color = Color::new(0.38, 0.38, 0.5, 1.0);
    /// Category color (lighter gray)
    pub const CATEGORY: Color = Color::new(0.6, 0.6, 0.7, 1.0);
    /// Description text color
    pub const DESCRIPTION: Color = Color::new(0.7, 0.7, 0.8, 1.0);
    /// Background particle color (subtle cyan)
    pub const BG_PARTICLE: Color = Color::new(0.0, 0.5, 0.7, 0.4);
    /// Background particle glow
    pub const BG_PARTICLE_GLOW: Color = Color::new(0.0, 0.6, 0.8, 0.15);
    /// Button border color (dim)
    pub const BUTTON_BORDER: Color = Color::new(0.25, 0.25, 0.35, 1.0);
    /// Button background (very dark, semi-transparent)
    pub const BUTTON_BG: Color = Color::new(0.05, 0.05, 0.12, 0.7);
    /// Particle core color (bright white-cyan)
    pub const PARTICLE_CORE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    /// Particle glow color (cyan)
    pub const PARTICLE_GLOW: Color = Color::new(0.0, 0.83, 1.0, 0.6);
}

/// Layout constants (designed for any resolution, scaled dynamically)
pub mod layout {
    /// Category bar Y position (percentage from top)
    pub const CATEGORY_Y_PERCENT: f32 = 0.25;
    /// Category spacing (percentage of screen width)
    pub const CATEGORY_SPACING_PERCENT: f32 = 0.35;
    /// Item list starting Y position (percentage from top)
    pub const ITEM_LIST_Y_PERCENT: f32 = 0.45;
    /// Item spacing (percentage of screen height)
    pub const ITEM_SPACING_PERCENT: f32 = 0.12;
    /// Description Y position (percentage from bottom)
    pub const DESCRIPTION_Y_PERCENT: f32 = 0.88;
    /// Category font size (percentage of screen height)
    pub const CATEGORY_FONT_PERCENT: f32 = 0.06;
    /// Item font size (percentage of screen height)
    pub const ITEM_FONT_PERCENT: f32 = 0.05;
    /// Description font size (percentage of screen height)
    pub const DESCRIPTION_FONT_PERCENT: f32 = 0.04;
}

/// Button style constants
pub mod button {
    /// Horizontal padding inside button
    pub const PADDING_H: f32 = 24.0;
    /// Vertical padding inside button
    pub const PADDING_V: f32 = 12.0;
    /// Corner radius for rounded rectangles
    pub const CORNER_RADIUS: f32 = 4.0;
    /// Border thickness
    pub const BORDER_WIDTH: f32 = 1.5;
    /// Number of orbiting particles
    pub const PARTICLE_COUNT: usize = 2;
    /// Particle orbit speed (full loops per second - lower = slower)
    pub const PARTICLE_SPEED: f32 = 0.08;
    /// Number of trail particles behind each main particle
    pub const TRAIL_COUNT: usize = 16;
    /// Trail spacing (percentage of perimeter between trail dots)
    pub const TRAIL_SPACING: f32 = 0.006;
    /// Particle size (radius)
    pub const PARTICLE_SIZE: f32 = 2.0;
    /// Glow size multiplier
    pub const GLOW_SIZE: f32 = 2.5;
}

/// Background particle constants (PS3-style floating dots)
pub mod bg_particles {
    /// Base particle size in pixels
    pub const BASE_SIZE: f32 = 3.0;
    /// Glow radius multiplier
    pub const GLOW_MULT: f32 = 3.0;
}

/// Convert a position along the rectangle perimeter (0.0-1.0) to x,y coordinates
/// Travels clockwise: top -> right -> bottom -> left
fn perimeter_to_xy(t: f32, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    let t = t.rem_euclid(1.0); // Wrap to 0-1
    let perimeter = 2.0 * w + 2.0 * h;
    let dist = t * perimeter;

    if dist < w {
        // Top edge (left to right)
        (x + dist, y)
    } else if dist < w + h {
        // Right edge (top to bottom)
        (x + w, y + (dist - w))
    } else if dist < 2.0 * w + h {
        // Bottom edge (right to left)
        (x + w - (dist - w - h), y + h)
    } else {
        // Left edge (bottom to top)
        (x, y + h - (dist - 2.0 * w - h))
    }
}

/// Draw a button with orbiting particles (for selected items)
fn draw_button_with_particles(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    time: f32,
    alpha: f32,
) {
    let r = button::CORNER_RADIUS;

    // Draw background
    draw_rounded_rect(x, y, w, h, r, Color::new(
        theme::BUTTON_BG.r,
        theme::BUTTON_BG.g,
        theme::BUTTON_BG.b,
        theme::BUTTON_BG.a * alpha,
    ));

    // Draw border
    draw_rounded_rect_lines(x, y, w, h, r, button::BORDER_WIDTH, Color::new(
        theme::SELECTED.r,
        theme::SELECTED.g,
        theme::SELECTED.b,
        alpha * 0.8,
    ));

    // Draw orbiting particles
    for i in 0..button::PARTICLE_COUNT {
        // Offset each particle evenly around the perimeter (opposite sides)
        let base_offset = i as f32 / button::PARTICLE_COUNT as f32;
        let particle_t = (base_offset + time * button::PARTICLE_SPEED).rem_euclid(1.0);

        // Draw trail first (behind the main particle)
        for trail_idx in (1..=button::TRAIL_COUNT).rev() {
            let trail_offset = trail_idx as f32 * button::TRAIL_SPACING;
            let trail_t = (particle_t - trail_offset).rem_euclid(1.0);
            let (tx, ty) = perimeter_to_xy(trail_t, x, y, w, h);

            // Fade out towards the tail
            let fade = 1.0 - (trail_idx as f32 / (button::TRAIL_COUNT as f32 + 1.0));
            let trail_alpha = alpha * fade * 0.7;
            let trail_size = button::PARTICLE_SIZE * (0.4 + fade * 0.6);

            // Trail dot with subtle glow
            draw_circle(tx, ty, trail_size * 1.5, Color::new(
                theme::PARTICLE_GLOW.r,
                theme::PARTICLE_GLOW.g,
                theme::PARTICLE_GLOW.b,
                trail_alpha * 0.4,
            ));
            draw_circle(tx, ty, trail_size, Color::new(
                theme::PARTICLE_CORE.r,
                theme::PARTICLE_CORE.g,
                theme::PARTICLE_CORE.b,
                trail_alpha,
            ));
        }

        // Main particle position
        let (px, py) = perimeter_to_xy(particle_t, x, y, w, h);

        // Outer glow
        draw_circle(px, py, button::PARTICLE_SIZE * button::GLOW_SIZE, Color::new(
            theme::PARTICLE_GLOW.r,
            theme::PARTICLE_GLOW.g,
            theme::PARTICLE_GLOW.b,
            alpha * 0.3,
        ));

        // Middle glow
        draw_circle(px, py, button::PARTICLE_SIZE * 1.5, Color::new(
            theme::PARTICLE_GLOW.r,
            theme::PARTICLE_GLOW.g,
            theme::PARTICLE_GLOW.b,
            alpha * 0.5,
        ));

        // Bright core
        draw_circle(px, py, button::PARTICLE_SIZE, Color::new(
            theme::PARTICLE_CORE.r,
            theme::PARTICLE_CORE.g,
            theme::PARTICLE_CORE.b,
            alpha,
        ));
    }
}

/// Draw an unselected button (simple border, no particles)
fn draw_button_unselected(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    alpha: f32,
) {
    let r = button::CORNER_RADIUS;

    // Draw subtle background
    draw_rounded_rect(x, y, w, h, r, Color::new(
        theme::BUTTON_BG.r,
        theme::BUTTON_BG.g,
        theme::BUTTON_BG.b,
        theme::BUTTON_BG.a * alpha * 0.5,
    ));

    // Draw dim border
    draw_rounded_rect_lines(x, y, w, h, r, button::BORDER_WIDTH, Color::new(
        theme::BUTTON_BORDER.r,
        theme::BUTTON_BORDER.g,
        theme::BUTTON_BORDER.b,
        alpha * 0.6,
    ));
}

/// Draw a rounded rectangle (filled)
fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    // Main body (without corners)
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);

    // Corner circles
    let segments = 8;
    draw_circle_segment(x + r, y + r, r, std::f32::consts::PI, std::f32::consts::FRAC_PI_2, segments, color); // Top-left
    draw_circle_segment(x + w - r, y + r, r, -std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, segments, color); // Top-right
    draw_circle_segment(x + w - r, y + h - r, r, 0.0, std::f32::consts::FRAC_PI_2, segments, color); // Bottom-right
    draw_circle_segment(x + r, y + h - r, r, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, segments, color); // Bottom-left
}

/// Draw a circle segment (quarter circle for rounded corners)
fn draw_circle_segment(cx: f32, cy: f32, r: f32, start_angle: f32, sweep: f32, segments: i32, color: Color) {
    for i in 0..segments {
        let a1 = start_angle + sweep * (i as f32 / segments as f32);
        let a2 = start_angle + sweep * ((i + 1) as f32 / segments as f32);

        let x1 = cx + a1.cos() * r;
        let y1 = cy + a1.sin() * r;
        let x2 = cx + a2.cos() * r;
        let y2 = cy + a2.sin() * r;

        draw_triangle(
            Vec2::new(cx, cy),
            Vec2::new(x1, y1),
            Vec2::new(x2, y2),
            color,
        );
    }
}

/// Draw rounded rectangle outline
fn draw_rounded_rect_lines(x: f32, y: f32, w: f32, h: f32, r: f32, thickness: f32, color: Color) {
    // Straight edges
    draw_line(x + r, y, x + w - r, y, thickness, color); // Top
    draw_line(x + w, y + r, x + w, y + h - r, thickness, color); // Right
    draw_line(x + w - r, y + h, x + r, y + h, thickness, color); // Bottom
    draw_line(x, y + h - r, x, y + r, thickness, color); // Left

    // Corner arcs
    let segments = 6;
    draw_arc(x + r, y + r, r, std::f32::consts::PI, std::f32::consts::FRAC_PI_2, segments, thickness, color); // Top-left
    draw_arc(x + w - r, y + r, r, -std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, segments, thickness, color); // Top-right
    draw_arc(x + w - r, y + h - r, r, 0.0, std::f32::consts::FRAC_PI_2, segments, thickness, color); // Bottom-right
    draw_arc(x + r, y + h - r, r, std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, segments, thickness, color); // Bottom-left
}

/// Draw an arc (for rounded corner outlines)
fn draw_arc(cx: f32, cy: f32, r: f32, start_angle: f32, sweep: f32, segments: i32, thickness: f32, color: Color) {
    for i in 0..segments {
        let a1 = start_angle + sweep * (i as f32 / segments as f32);
        let a2 = start_angle + sweep * ((i + 1) as f32 / segments as f32);

        let x1 = cx + a1.cos() * r;
        let y1 = cy + a1.sin() * r;
        let x2 = cx + a2.cos() * r;
        let y2 = cy + a2.sin() * r;

        draw_line(x1, y1, x2, y2, thickness, color);
    }
}

/// Draw the XMB menu (renders directly to screen for crisp text)
pub fn draw_xmb(state: &XMBState) {
    draw_xmb_with_font(state, None);
}

/// Draw the XMB menu with a custom font
pub fn draw_xmb_with_font(state: &XMBState, font: XMBFont) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    // 1. Draw background gradient
    draw_gradient_background(screen_w, screen_h);

    // 2. Draw PS3-style floating particles
    draw_bg_particles(state, screen_w, screen_h);

    // 3. Draw category bar (horizontal)
    draw_category_bar(state, screen_w, screen_h, font.as_ref());

    // 4. Draw item list (vertical)
    draw_item_list(state, screen_w, screen_h, font.as_ref());

    // 5. Draw description at bottom
    draw_description(state, screen_w, screen_h, font.as_ref());

    // 6. Draw status message if present
    draw_status_message(state, screen_w, screen_h, font.as_ref());
}

/// Draw vertical gradient background
fn draw_gradient_background(screen_w: f32, screen_h: f32) {
    // Split screen into horizontal strips for gradient effect
    let strips = 10;
    let strip_height = screen_h / strips as f32;

    for i in 0..strips {
        let t = i as f32 / strips as f32;
        let color = Color::new(
            theme::BG_TOP.r * (1.0 - t) + theme::BG_BOTTOM.r * t,
            theme::BG_TOP.g * (1.0 - t) + theme::BG_BOTTOM.g * t,
            theme::BG_TOP.b * (1.0 - t) + theme::BG_BOTTOM.b * t,
            1.0,
        );

        let y = i as f32 * strip_height;
        draw_rectangle(0.0, y, screen_w, strip_height, color);
    }
}

/// Draw PS3-style floating background particles
fn draw_bg_particles(state: &XMBState, screen_w: f32, screen_h: f32) {
    for particle in &state.bg_particles {
        // Convert normalized position to screen coordinates
        let base_x = particle.x * screen_w;
        let base_y = particle.y * screen_h;

        // Add orbital motion around the base position
        let orbit_x = particle.angle.cos() * particle.orbit_radius * screen_w * 0.5;
        let orbit_y = particle.angle.sin() * particle.orbit_radius * screen_h * 0.5;

        let px = base_x + orbit_x;
        let py = base_y + orbit_y;

        // Calculate size based on particle properties
        let size = bg_particles::BASE_SIZE * particle.size;
        let glow_size = size * bg_particles::GLOW_MULT;

        // Draw outer glow
        draw_circle(px, py, glow_size, Color::new(
            theme::BG_PARTICLE_GLOW.r,
            theme::BG_PARTICLE_GLOW.g,
            theme::BG_PARTICLE_GLOW.b,
            theme::BG_PARTICLE_GLOW.a * particle.alpha,
        ));

        // Draw middle glow
        draw_circle(px, py, size * 1.5, Color::new(
            theme::BG_PARTICLE.r,
            theme::BG_PARTICLE.g,
            theme::BG_PARTICLE.b,
            particle.alpha * 0.3,
        ));

        // Draw core
        draw_circle(px, py, size, Color::new(
            theme::BG_PARTICLE.r,
            theme::BG_PARTICLE.g,
            theme::BG_PARTICLE.b,
            particle.alpha * 0.6,
        ));
    }
}

/// Draw the horizontal category bar with button styling
fn draw_category_bar(state: &XMBState, screen_w: f32, screen_h: f32, font: Option<&Font>) {
    let center_x = screen_w / 2.0;
    let y = screen_h * layout::CATEGORY_Y_PERCENT;
    let spacing = screen_w * layout::CATEGORY_SPACING_PERCENT;
    let font_size = (screen_h * layout::CATEGORY_FONT_PERCENT).max(12.0) as u16;

    for (idx, category) in state.categories.iter().enumerate() {
        let offset_from_selected = idx as f32 - state.category_scroll;
        let x = center_x + offset_from_selected * spacing;

        // Calculate alpha based on distance from center
        let distance = offset_from_selected.abs();
        let alpha = (1.0 - (distance * 0.5).min(1.0)).max(0.0);

        // Skip if too far away
        if alpha <= 0.0 {
            continue;
        }

        let is_selected = idx == state.selected_category;

        // Measure text for button sizing
        let text_dims = measure_text(&category.label, font, font_size, 1.0);
        let btn_w = text_dims.width + button::PADDING_H * 2.0;
        let btn_h = text_dims.height + button::PADDING_V * 2.0;
        let btn_x = x - btn_w / 2.0;
        let btn_y = y - text_dims.height - button::PADDING_V;

        // Draw button (with or without particles)
        if is_selected {
            draw_button_with_particles(btn_x, btn_y, btn_w, btn_h, state.time, alpha);
        } else {
            draw_button_unselected(btn_x, btn_y, btn_w, btn_h, alpha);
        }

        // Text color
        let color = if is_selected {
            Color::new(theme::SELECTED.r, theme::SELECTED.g, theme::SELECTED.b, alpha)
        } else {
            Color::new(theme::CATEGORY.r, theme::CATEGORY.g, theme::CATEGORY.b, alpha * 0.7)
        };

        // Center the text inside button
        let text_x = x - text_dims.width / 2.0;

        draw_text_ex(
            &category.label,
            text_x,
            y,
            TextParams {
                font_size,
                font,
                color,
                ..Default::default()
            },
        );
    }
}

/// Draw the vertical item list with button styling
fn draw_item_list(state: &XMBState, screen_w: f32, screen_h: f32, font: Option<&Font>) {
    if let Some(category) = state.get_selected_category() {
        let center_x = screen_w / 2.0;
        let base_y = screen_h * layout::ITEM_LIST_Y_PERCENT;
        let spacing = screen_h * layout::ITEM_SPACING_PERCENT;
        let font_size = (screen_h * layout::ITEM_FONT_PERCENT).max(10.0) as u16;

        for (idx, item) in category.items.iter().enumerate() {
            let offset_from_selected = idx as f32 - state.item_scroll;
            let y = base_y + offset_from_selected * spacing;

            // Calculate alpha based on distance from selected
            let distance = offset_from_selected.abs();
            let alpha = (1.0 - (distance * 0.6).min(1.0)).max(0.0);

            // Skip if too far away
            if alpha <= 0.0 {
                continue;
            }

            let is_selected = idx == state.selected_item;

            // Measure text for button sizing
            let text_dims = measure_text(&item.label, font, font_size, 1.0);
            let btn_w = text_dims.width + button::PADDING_H * 2.0;
            let btn_h = text_dims.height + button::PADDING_V * 2.0;
            let btn_x = center_x - btn_w / 2.0;
            let btn_y = y - text_dims.height - button::PADDING_V;

            // Draw button (with or without particles)
            if is_selected {
                draw_button_with_particles(btn_x, btn_y, btn_w, btn_h, state.time, alpha);
            } else {
                draw_button_unselected(btn_x, btn_y, btn_w, btn_h, alpha);
            }

            // Text color
            let color = if is_selected {
                Color::new(theme::SELECTED.r, theme::SELECTED.g, theme::SELECTED.b, alpha)
            } else {
                Color::new(theme::UNSELECTED.r, theme::UNSELECTED.g, theme::UNSELECTED.b, alpha * 0.8)
            };

            // Center the text inside button
            let text_x = center_x - text_dims.width / 2.0;

            draw_text_ex(
                &item.label,
                text_x,
                y,
                TextParams {
                    font_size,
                    font,
                    color,
                    ..Default::default()
                },
            );
        }
    }
}

/// Draw description text at bottom
fn draw_description(state: &XMBState, screen_w: f32, screen_h: f32, font: Option<&Font>) {
    if let Some(description) = state.get_selected_description() {
        let center_x = screen_w / 2.0;
        let y = screen_h * layout::DESCRIPTION_Y_PERCENT;
        let font_size = (screen_h * layout::DESCRIPTION_FONT_PERCENT).max(10.0) as u16;

        // Center the description text
        let text_dims = measure_text(description, font, font_size, 1.0);
        let text_x = center_x - text_dims.width / 2.0;

        draw_text_ex(
            description,
            text_x,
            y,
            TextParams {
                font_size,
                font,
                color: theme::DESCRIPTION,
                ..Default::default()
            },
        );
    }
}

/// Draw status message (centered, temporary notification)
fn draw_status_message(state: &XMBState, screen_w: f32, screen_h: f32, font: Option<&Font>) {
    if let Some(message) = &state.status_message {
        let center_x = screen_w / 2.0;
        let center_y = screen_h / 2.0;
        let font_size = (screen_h * 0.05).max(16.0) as u16;

        // Measure text for background box
        let text_dims = measure_text(message, font, font_size, 1.0);
        let padding = 20.0;
        let box_w = text_dims.width + padding * 2.0;
        let box_h = text_dims.height + padding * 2.0;
        let box_x = center_x - box_w / 2.0;
        let box_y = center_y - box_h / 2.0;

        // Fade based on remaining time (fade out in last 0.5 seconds)
        let alpha = (state.status_timer / 0.5).min(1.0);

        // Draw semi-transparent background
        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            Color::new(0.0, 0.0, 0.0, 0.8 * alpha),
        );

        // Draw border
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            2.0,
            Color::new(theme::SELECTED.r, theme::SELECTED.g, theme::SELECTED.b, alpha),
        );

        // Draw text centered in box
        let text_x = center_x - text_dims.width / 2.0;
        let text_y = center_y + text_dims.height / 4.0;

        draw_text_ex(
            message,
            text_x,
            text_y,
            TextParams {
                font_size,
                font,
                color: Color::new(1.0, 1.0, 1.0, alpha),
                ..Default::default()
            },
        );
    }
}
