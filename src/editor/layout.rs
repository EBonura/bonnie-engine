//! Editor layout - TRLE-inspired panel arrangement

use macroquad::prelude::*;
use crate::ui::{Rect, UiContext, SplitPanel, draw_panel, panel_content_rect, Toolbar, icon};
use crate::rasterizer::{Framebuffer, Texture as RasterTexture};
use super::{EditorState, EditorTool};
use super::grid_view::draw_grid_view;
use super::viewport_3d::draw_viewport_3d;
use super::texture_palette::draw_texture_palette;

/// Actions that can be triggered by the editor UI
#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    None,
    Play,
    New,
    Save,
    SaveAs,
    Load(String), // Path to load
    PromptLoad,   // Show file prompt
    Export,       // Browser: download as file
    Import,       // Browser: upload file
    Exit,         // Close/quit
}

/// Editor layout state (split panel ratios)
pub struct EditorLayout {
    /// Main horizontal split (left panels | center+right)
    pub main_split: SplitPanel,
    /// Right split (center viewport | right panels)
    pub right_split: SplitPanel,
    /// Left vertical split (2D grid | room properties)
    pub left_split: SplitPanel,
    /// Right vertical split (texture palette | properties)
    pub right_panel_split: SplitPanel,
}

impl EditorLayout {
    pub fn new() -> Self {
        Self {
            main_split: SplitPanel::horizontal(1).with_ratio(0.25).with_min_size(150.0),
            right_split: SplitPanel::horizontal(2).with_ratio(0.75).with_min_size(150.0),
            left_split: SplitPanel::vertical(3).with_ratio(0.6).with_min_size(100.0),
            right_panel_split: SplitPanel::vertical(4).with_ratio(0.6).with_min_size(100.0),
        }
    }

    /// Apply layout config from a level
    pub fn apply_config(&mut self, config: &crate::world::EditorLayoutConfig) {
        self.main_split.ratio = config.main_split;
        self.right_split.ratio = config.right_split;
        self.left_split.ratio = config.left_split;
        self.right_panel_split.ratio = config.right_panel_split;
    }

    /// Extract current layout as a config (for saving with level)
    pub fn to_config(&self) -> crate::world::EditorLayoutConfig {
        crate::world::EditorLayoutConfig {
            main_split: self.main_split.ratio,
            right_split: self.right_split.ratio,
            left_split: self.left_split.ratio,
            right_panel_split: self.right_panel_split.ratio,
        }
    }
}

/// Draw the complete editor UI, returns action if triggered
pub fn draw_editor(
    ctx: &mut UiContext,
    layout: &mut EditorLayout,
    state: &mut EditorState,
    textures: &[RasterTexture],
    fb: &mut Framebuffer,
    bounds: Rect,
    icon_font: Option<&Font>,
) -> EditorAction {
    let screen = bounds;

    // Single unified toolbar at top
    let toolbar_height = 36.0;
    let toolbar_rect = screen.slice_top(toolbar_height);
    let main_rect = screen.remaining_after_top(toolbar_height);

    // Status bar at bottom
    let status_height = 22.0;
    let status_rect = main_rect.slice_bottom(status_height);
    let panels_rect = main_rect.remaining_after_bottom(status_height);

    // Draw unified toolbar
    let action = draw_unified_toolbar(ctx, toolbar_rect, state, icon_font);

    // Main split: left panels | rest
    let (left_rect, rest_rect) = layout.main_split.update(ctx, panels_rect);

    // Right split: center viewport | right panels
    let (center_rect, right_rect) = layout.right_split.update(ctx, rest_rect);

    // Left split: 2D grid view | room controls
    let (grid_rect, room_props_rect) = layout.left_split.update(ctx, left_rect);

    // Right split: texture palette | face properties
    let (texture_rect, props_rect) = layout.right_panel_split.update(ctx, right_rect);

    // Draw panels
    draw_panel(grid_rect, Some("2D Grid"), Color::from_rgba(35, 35, 40, 255));
    draw_grid_view(ctx, panel_content_rect(grid_rect, true), state);

    draw_panel(room_props_rect, Some("Room"), Color::from_rgba(35, 35, 40, 255));
    draw_room_properties(ctx, panel_content_rect(room_props_rect, true), state);

    draw_panel(center_rect, Some("3D Viewport"), Color::from_rgba(25, 25, 30, 255));
    draw_viewport_3d(ctx, panel_content_rect(center_rect, true), state, textures, fb);

    draw_panel(texture_rect, Some("Textures"), Color::from_rgba(35, 35, 40, 255));
    draw_texture_palette(ctx, panel_content_rect(texture_rect, true), state, icon_font);

    draw_panel(props_rect, Some("Properties"), Color::from_rgba(35, 35, 40, 255));
    draw_properties(ctx, panel_content_rect(props_rect, true), state);

    // Draw status bar
    draw_status_bar(status_rect, state);

    action
}

fn draw_unified_toolbar(ctx: &mut UiContext, rect: Rect, state: &mut EditorState, icon_font: Option<&Font>) -> EditorAction {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::from_rgba(40, 40, 45, 255));

    let mut action = EditorAction::None;
    let mut toolbar = Toolbar::new(rect);

    // File operations
    if toolbar.icon_button(ctx, icon::FILE_PLUS, icon_font, "New") {
        action = EditorAction::New;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if toolbar.icon_button(ctx, icon::FOLDER_OPEN, icon_font, "Open") {
            action = EditorAction::PromptLoad;
        }
        if toolbar.icon_button(ctx, icon::SAVE, icon_font, "Save") {
            action = EditorAction::Save;
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if toolbar.icon_button(ctx, icon::FOLDER_OPEN, icon_font, "Upload") {
            action = EditorAction::Import;
        }
        if toolbar.icon_button(ctx, icon::SAVE, icon_font, "Download") {
            action = EditorAction::Export;
        }
    }

    toolbar.separator();

    // Edit operations
    if toolbar.icon_button(ctx, icon::UNDO, icon_font, "Undo") {
        state.undo();
    }
    if toolbar.icon_button(ctx, icon::REDO, icon_font, "Redo") {
        state.redo();
    }

    toolbar.separator();

    // Play button
    if toolbar.icon_button(ctx, icon::PLAY, icon_font, "Play") {
        action = EditorAction::Play;
    }

    toolbar.separator();

    // Tool buttons
    let tools = [
        (icon::MOVE, "Select", EditorTool::Select),
        (icon::SQUARE, "Floor", EditorTool::DrawFloor),
        (icon::BOX, "Wall", EditorTool::DrawWall),
        (icon::LAYERS, "Ceiling", EditorTool::DrawCeiling),
        (icon::DOOR_CLOSED, "Portal", EditorTool::PlacePortal),
    ];

    for (icon_char, tooltip, tool) in tools {
        let is_active = state.tool == tool;
        if toolbar.icon_button_active(ctx, icon_char, icon_font, tooltip, is_active) {
            state.tool = tool;
        }
    }

    toolbar.separator();

    // Vertex mode toggle
    let link_icon = if state.link_coincident_vertices { icon::LINK } else { icon::UNLINK };
    let link_tooltip = if state.link_coincident_vertices { "Vertices Linked" } else { "Vertices Independent" };
    if toolbar.icon_button_active(ctx, link_icon, icon_font, link_tooltip, state.link_coincident_vertices) {
        state.link_coincident_vertices = !state.link_coincident_vertices;
        let mode = if state.link_coincident_vertices { "Linked" } else { "Independent" };
        state.set_status(&format!("Vertex mode: {}", mode), 2.0);
    }

    toolbar.separator();

    // Room navigation
    toolbar.label(&format!("Room: {}", state.current_room));

    if toolbar.icon_button(ctx, icon::CIRCLE_CHEVRON_LEFT, icon_font, "Previous Room") {
        if state.current_room > 0 {
            state.current_room -= 1;
        }
    }
    if toolbar.icon_button(ctx, icon::CIRCLE_CHEVRON_RIGHT, icon_font, "Next Room") {
        if state.current_room + 1 < state.level.rooms.len() {
            state.current_room += 1;
        }
    }
    if toolbar.icon_button(ctx, icon::PLUS, icon_font, "Add Room") {
        // TODO: Add new room
        println!("Add room clicked");
    }

    toolbar.separator();

    // PS1 effect toggles
    if toolbar.icon_button_active(ctx, icon::WAVES, icon_font, "Affine Textures (PS1 warp)", state.raster_settings.affine_textures) {
        state.raster_settings.affine_textures = !state.raster_settings.affine_textures;
        let mode = if state.raster_settings.affine_textures { "ON" } else { "OFF" };
        state.set_status(&format!("Affine textures: {}", mode), 2.0);
    }
    if toolbar.icon_button_active(ctx, icon::MAGNET, icon_font, "Vertex Snap (PS1 jitter)", state.raster_settings.vertex_snap) {
        state.raster_settings.vertex_snap = !state.raster_settings.vertex_snap;
        let mode = if state.raster_settings.vertex_snap { "ON" } else { "OFF" };
        state.set_status(&format!("Vertex snap: {}", mode), 2.0);
    }
    if toolbar.icon_button_active(ctx, icon::SUN, icon_font, "Gouraud Shading", state.raster_settings.shading != crate::rasterizer::ShadingMode::None) {
        use crate::rasterizer::ShadingMode;
        state.raster_settings.shading = if state.raster_settings.shading == ShadingMode::None {
            ShadingMode::Gouraud
        } else {
            ShadingMode::None
        };
        let mode = if state.raster_settings.shading != ShadingMode::None { "ON" } else { "OFF" };
        state.set_status(&format!("Shading: {}", mode), 2.0);
    }
    if toolbar.icon_button_active(ctx, icon::MONITOR, icon_font, "Low Resolution (PS1 320x240)", state.raster_settings.low_resolution) {
        state.raster_settings.low_resolution = !state.raster_settings.low_resolution;
        let mode = if state.raster_settings.low_resolution { "320x240" } else { "High-res" };
        state.set_status(&format!("Resolution: {}", mode), 2.0);
    }
    if toolbar.icon_button_active(ctx, icon::BLEND, icon_font, "Dithering (PS1 color banding)", state.raster_settings.dithering) {
        state.raster_settings.dithering = !state.raster_settings.dithering;
        let mode = if state.raster_settings.dithering { "ON" } else { "OFF" };
        state.set_status(&format!("Dithering: {}", mode), 2.0);
    }

    toolbar.separator();

    // Current file label
    let file_label = match &state.current_file {
        Some(path) => {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string());
            if state.dirty {
                format!("{}*", name)
            } else {
                name
            }
        }
        None => {
            if state.dirty {
                "untitled*".to_string()
            } else {
                "untitled".to_string()
            }
        }
    };
    toolbar.label(&file_label);

    // Keyboard shortcuts
    let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
             || is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

    if ctrl && is_key_pressed(KeyCode::N) {
        action = EditorAction::New;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if ctrl && is_key_pressed(KeyCode::O) {
            action = EditorAction::PromptLoad;
        }
        if ctrl && shift && is_key_pressed(KeyCode::S) {
            action = EditorAction::SaveAs;
        } else if ctrl && is_key_pressed(KeyCode::S) {
            action = EditorAction::Save;
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if ctrl && is_key_pressed(KeyCode::O) {
            action = EditorAction::Import;
        }
        if ctrl && is_key_pressed(KeyCode::S) {
            action = EditorAction::Export;
        }
    }
    if ctrl && is_key_pressed(KeyCode::Z) {
        if shift {
            state.redo();
        } else {
            state.undo();
        }
    }

    action
}

fn draw_room_properties(ctx: &mut UiContext, rect: Rect, state: &mut EditorState) {
    let mut y = rect.y.floor();
    let x = rect.x.floor();
    let line_height = 20.0;

    if let Some(room) = state.current_room() {
        draw_text(&format!("ID: {}", room.id), x, (y + 14.0).floor(), 16.0, WHITE);
        y += line_height;

        draw_text(
            &format!("Pos: ({:.1}, {:.1}, {:.1})", room.position.x, room.position.y, room.position.z),
            x, (y + 14.0).floor(), 16.0, WHITE,
        );
        y += line_height;

        // Count sectors
        let sector_count = room.iter_sectors().count();
        draw_text(&format!("Size: {}x{}", room.width, room.depth), x, (y + 14.0).floor(), 16.0, WHITE);
        y += line_height;

        draw_text(&format!("Sectors: {}", sector_count), x, (y + 14.0).floor(), 16.0, WHITE);
        y += line_height;

        draw_text(&format!("Portals: {}", room.portals.len()), x, (y + 14.0).floor(), 16.0, WHITE);
        y += line_height;

        // Room list
        y += 10.0;
        draw_text("Rooms:", x, (y + 14.0).floor(), 16.0, Color::from_rgba(150, 150, 150, 255));
        y += line_height;

        for (i, room) in state.level.rooms.iter().enumerate() {
            let is_selected = i == state.current_room;
            let color = if is_selected {
                Color::from_rgba(100, 200, 100, 255)
            } else {
                WHITE
            };

            let room_btn_rect = Rect::new(x, y, rect.w - 4.0, line_height);
            if ctx.mouse.clicked(&room_btn_rect) {
                state.current_room = i;
            }

            if is_selected {
                draw_rectangle(room_btn_rect.x.floor(), room_btn_rect.y.floor(), room_btn_rect.w, room_btn_rect.h, Color::from_rgba(60, 80, 60, 255));
            }

            let sector_count = room.iter_sectors().count();
            draw_text(&format!("  Room {} ({} sectors)", room.id, sector_count), x, (y + 14.0).floor(), 16.0, color);
            y += line_height;

            if y > rect.bottom() - line_height {
                break;
            }
        }
    } else {
        draw_text("No room selected", x, (y + 14.0).floor(), 16.0, Color::from_rgba(150, 150, 150, 255));
    }
}

fn draw_properties(ctx: &mut UiContext, rect: Rect, state: &mut EditorState) {
    let mut y = rect.y.floor();
    let x = rect.x.floor();
    let line_height = 20.0;
    let checkbox_size = 14.0;
    let indent = 10.0;

    // Helper to draw a checkbox toggle
    let draw_checkbox = |ctx: &mut UiContext, cx: f32, cy: f32, checked: bool, label: &str| -> bool {
        let box_rect = Rect::new(cx, cy, checkbox_size, checkbox_size);
        let hovered = ctx.mouse.inside(&box_rect);
        let clicked = ctx.mouse.clicked(&box_rect);

        // Draw checkbox box
        let box_color = if hovered {
            Color::from_rgba(80, 80, 90, 255)
        } else {
            Color::from_rgba(50, 50, 60, 255)
        };
        draw_rectangle(cx.floor(), cy.floor(), checkbox_size, checkbox_size, box_color);
        draw_rectangle_lines(cx.floor(), cy.floor(), checkbox_size, checkbox_size, 1.0, Color::from_rgba(100, 100, 110, 255));

        // Draw checkmark if checked
        if checked {
            draw_text("✓", (cx + 2.0).floor(), (cy + 12.0).floor(), 14.0, Color::from_rgba(100, 255, 100, 255));
        }

        // Draw label
        draw_text(label, (cx + checkbox_size + 5.0).floor(), (cy + 12.0).floor(), 14.0, WHITE);

        clicked
    };

    // Clone selection to avoid borrow issues
    let selection = state.selection.clone();

    match &selection {
        super::Selection::None => {
            draw_text("Nothing selected", x, (y + 14.0).floor(), 16.0, Color::from_rgba(150, 150, 150, 255));
        }
        super::Selection::Room(idx) => {
            draw_text(&format!("Room {}", idx), x, (y + 14.0).floor(), 16.0, WHITE);
        }
        super::Selection::Sector { room, x: gx, z: gz } | super::Selection::SectorFace { room, x: gx, z: gz, .. } => {
            // Header
            draw_text(&format!("Sector ({}, {})", gx, gz), x, (y + 14.0).floor(), 16.0, Color::from_rgba(255, 200, 80, 255));
            y += line_height;

            // Get sector data
            let sector_data = state.level.rooms.get(*room)
                .and_then(|r| r.get_sector(*gx, *gz))
                .cloned();

            if let Some(sector) = sector_data {
                // === FLOOR ===
                y += 5.0; // spacing
                draw_text("Floor", x, (y + 14.0).floor(), 14.0, Color::from_rgba(150, 200, 255, 255));
                y += line_height;

                if let Some(floor) = &sector.floor {
                    // Texture
                    let tex_display = if floor.texture.is_valid() {
                        format!("{}", floor.texture.name)
                    } else {
                        String::from("(none)")
                    };
                    draw_text(&format!("  Tex: {}", tex_display), x, (y + 14.0).floor(), 14.0, WHITE);
                    y += line_height;

                    // Heights (show if sloped)
                    if !floor.is_flat() {
                        draw_text(&format!("  Heights: [{:.0}, {:.0}, {:.0}, {:.0}]",
                            floor.heights[0], floor.heights[1], floor.heights[2], floor.heights[3]),
                            x, (y + 14.0).floor(), 14.0, WHITE);
                        y += line_height;
                    } else {
                        draw_text(&format!("  Height: {:.0}", floor.heights[0]), x, (y + 14.0).floor(), 14.0, WHITE);
                        y += line_height;
                    }

                    // Walkable toggle
                    let walkable = floor.walkable;
                    if draw_checkbox(ctx, x + indent, y, walkable, "Walkable") {
                        if let Some(r) = state.level.rooms.get_mut(*room) {
                            if let Some(s) = r.get_sector_mut(*gx, *gz) {
                                if let Some(f) = &mut s.floor {
                                    f.walkable = !f.walkable;
                                }
                            }
                        }
                    }
                    y += line_height;

                    // Blend mode
                    draw_text(&format!("  Blend: {:?}", floor.blend_mode), x, (y + 14.0).floor(), 14.0, Color::from_rgba(150, 150, 150, 255));
                    y += line_height;
                } else {
                    draw_text("  (no floor)", x, (y + 14.0).floor(), 14.0, Color::from_rgba(100, 100, 100, 255));
                    y += line_height;
                }

                // === CEILING ===
                y += 5.0;
                draw_text("Ceiling", x, (y + 14.0).floor(), 14.0, Color::from_rgba(200, 150, 255, 255));
                y += line_height;

                if let Some(ceiling) = &sector.ceiling {
                    // Texture
                    let tex_display = if ceiling.texture.is_valid() {
                        format!("{}", ceiling.texture.name)
                    } else {
                        String::from("(none)")
                    };
                    draw_text(&format!("  Tex: {}", tex_display), x, (y + 14.0).floor(), 14.0, WHITE);
                    y += line_height;

                    // Height
                    if !ceiling.is_flat() {
                        draw_text(&format!("  Heights: [{:.0}, {:.0}, {:.0}, {:.0}]",
                            ceiling.heights[0], ceiling.heights[1], ceiling.heights[2], ceiling.heights[3]),
                            x, (y + 14.0).floor(), 14.0, WHITE);
                    } else {
                        draw_text(&format!("  Height: {:.0}", ceiling.heights[0]), x, (y + 14.0).floor(), 14.0, WHITE);
                    }
                    y += line_height;

                    // Walkable toggle (for ceiling, usually false)
                    let walkable = ceiling.walkable;
                    if draw_checkbox(ctx, x + indent, y, walkable, "Walkable") {
                        if let Some(r) = state.level.rooms.get_mut(*room) {
                            if let Some(s) = r.get_sector_mut(*gx, *gz) {
                                if let Some(c) = &mut s.ceiling {
                                    c.walkable = !c.walkable;
                                }
                            }
                        }
                    }
                    y += line_height;
                } else {
                    draw_text("  (no ceiling)", x, (y + 14.0).floor(), 14.0, Color::from_rgba(100, 100, 100, 255));
                    y += line_height;
                }

                // === WALLS ===
                y += 5.0;
                let wall_count = sector.walls_north.len() + sector.walls_east.len()
                    + sector.walls_south.len() + sector.walls_west.len();
                draw_text(&format!("Walls ({})", wall_count), x, (y + 14.0).floor(), 14.0, Color::from_rgba(255, 180, 120, 255));
                y += line_height;

                // Show wall details per direction
                let wall_dirs = [
                    ("N", &sector.walls_north),
                    ("E", &sector.walls_east),
                    ("S", &sector.walls_south),
                    ("W", &sector.walls_west),
                ];

                for (dir_name, walls) in wall_dirs {
                    if !walls.is_empty() {
                        for (i, wall) in walls.iter().enumerate() {
                            let label = if walls.len() == 1 {
                                format!("  {}: {:.0}-{:.0}", dir_name, wall.y_bottom, wall.y_top)
                            } else {
                                format!("  {}[{}]: {:.0}-{:.0}", dir_name, i, wall.y_bottom, wall.y_top)
                            };
                            draw_text(&label, x, (y + 14.0).floor(), 14.0, WHITE);
                            y += line_height;

                            if y > rect.bottom() - line_height * 2.0 {
                                draw_text("  ...", x, (y + 14.0).floor(), 14.0, Color::from_rgba(100, 100, 100, 255));
                                return; // Out of space
                            }
                        }
                    }
                }
            } else {
                draw_text("  Sector not found", x, (y + 14.0).floor(), 14.0, Color::from_rgba(255, 100, 100, 255));
            }
        }
        super::Selection::Portal { room, portal } => {
            draw_text(&format!("Portal {} in Room {}", portal, room), x, (y + 14.0).floor(), 16.0, WHITE);
        }
    }
}

fn draw_status_bar(rect: Rect, state: &EditorState) {
    draw_rectangle(rect.x.floor(), rect.y.floor(), rect.w, rect.h, Color::from_rgba(40, 40, 45, 255));

    // Show status message in center if available
    if let Some(msg) = state.get_status() {
        let msg_width = msg.len() as f32 * 8.0;
        let center_x = rect.x + rect.w * 0.5 - msg_width * 0.5;
        draw_text(&msg, center_x.floor(), (rect.y + 15.0).floor(), 16.0, Color::from_rgba(100, 255, 100, 255));
    }

    // Show keyboard shortcuts hint on the right (platform-specific)
    #[cfg(not(target_arch = "wasm32"))]
    let hints = "Ctrl+S: Save | Ctrl+Shift+S: Save As | Ctrl+O: Open | Ctrl+N: New";
    #[cfg(target_arch = "wasm32")]
    let hints = "Ctrl+S: Download | Ctrl+O: Upload | Ctrl+N: New";

    let hint_width = hints.len() as f32 * 6.0; // Approximate width
    draw_text(
        hints,
        (rect.right() - hint_width - 8.0).floor(),
        (rect.y + 15.0).floor(),
        14.0,
        Color::from_rgba(100, 100, 100, 255),
    );
}
