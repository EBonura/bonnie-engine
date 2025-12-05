//! XMB State Management
//!
//! Manages the current state of the XMB menu including selection and animations

use super::menu::{XMBAction, XMBCategory, create_default_menu};

/// Number of background particles (PS3-style floating dots)
pub const BG_PARTICLE_COUNT: usize = 40;

/// A single background particle with position, velocity and properties
#[derive(Clone, Copy)]
pub struct BgParticle {
    /// X position (0.0 to 1.0 normalized)
    pub x: f32,
    /// Y position (0.0 to 1.0 normalized)
    pub y: f32,
    /// X velocity (normalized per second)
    pub vx: f32,
    /// Y velocity (normalized per second)
    pub vy: f32,
    /// Base rotation angle
    pub angle: f32,
    /// Angular velocity (radians per second)
    pub angular_vel: f32,
    /// Orbit radius (for circular motion)
    pub orbit_radius: f32,
    /// Size multiplier (0.5 to 1.5)
    pub size: f32,
    /// Alpha multiplier (0.3 to 1.0)
    pub alpha: f32,
    /// Phase offset for orbit
    pub phase: f32,
}

impl BgParticle {
    /// Create a new particle with random properties
    pub fn new_random(seed: u32) -> Self {
        // Simple pseudo-random based on seed
        let hash = |s: u32| -> f32 {
            let x = s.wrapping_mul(2654435761);
            (x as f32 / u32::MAX as f32)
        };

        Self {
            x: hash(seed),
            y: hash(seed.wrapping_add(1)),
            vx: (hash(seed.wrapping_add(2)) - 0.5) * 0.02,
            vy: (hash(seed.wrapping_add(3)) - 0.5) * 0.02,
            angle: hash(seed.wrapping_add(4)) * std::f32::consts::TAU,
            angular_vel: (hash(seed.wrapping_add(5)) - 0.5) * 0.5,
            orbit_radius: hash(seed.wrapping_add(6)) * 0.03 + 0.01,
            size: hash(seed.wrapping_add(7)) * 1.0 + 0.5,
            alpha: hash(seed.wrapping_add(8)) * 0.5 + 0.3,
            phase: hash(seed.wrapping_add(9)) * std::f32::consts::TAU,
        }
    }
}

/// XMB menu state with selection tracking and animation values
pub struct XMBState {
    /// All menu categories
    pub categories: Vec<XMBCategory>,
    /// Currently selected category index
    pub selected_category: usize,
    /// Currently selected item index within category
    pub selected_item: usize,
    /// Horizontal scroll animation value (0.0 = leftmost category)
    pub category_scroll: f32,
    /// Vertical scroll animation value (0.0 = topmost item)
    pub item_scroll: f32,
    /// Time accumulator for animations (in seconds)
    pub time: f32,
    /// Selection pulse animation (0.0 to 1.0)
    pub pulse: f32,
    /// Status message to display (e.g., "Not yet implemented")
    pub status_message: Option<String>,
    /// Time remaining to show status message
    pub status_timer: f32,
    /// Background particles (PS3-style)
    pub bg_particles: Vec<BgParticle>,
    /// Velocity impulse from navigation (decays over time)
    pub nav_impulse_x: f32,
    pub nav_impulse_y: f32,
}

impl XMBState {
    /// Create a new XMB state with default menu
    pub fn new() -> Self {
        // Initialize background particles
        let bg_particles: Vec<BgParticle> = (0..BG_PARTICLE_COUNT)
            .map(|i| BgParticle::new_random(i as u32 * 31337))
            .collect();

        Self {
            categories: create_default_menu(),
            selected_category: 0,
            selected_item: 0,
            category_scroll: 0.0,
            item_scroll: 0.0,
            time: 0.0,
            pulse: 0.0,
            status_message: None,
            status_timer: 0.0,
            bg_particles,
            nav_impulse_x: 0.0,
            nav_impulse_y: 0.0,
        }
    }

    /// Create XMB state with custom categories
    pub fn with_categories(categories: Vec<XMBCategory>) -> Self {
        let bg_particles: Vec<BgParticle> = (0..BG_PARTICLE_COUNT)
            .map(|i| BgParticle::new_random(i as u32 * 31337))
            .collect();

        Self {
            categories,
            selected_category: 0,
            selected_item: 0,
            category_scroll: 0.0,
            item_scroll: 0.0,
            time: 0.0,
            pulse: 0.0,
            status_message: None,
            status_timer: 0.0,
            bg_particles,
            nav_impulse_x: 0.0,
            nav_impulse_y: 0.0,
        }
    }

    /// Set a status message to display temporarily
    pub fn set_status(&mut self, message: &str, duration: f32) {
        self.status_message = Some(message.to_string());
        self.status_timer = duration;
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_timer = 0.0;
    }

    /// Update animations (call once per frame with delta time)
    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Update pulse animation (sine wave 0.0 to 1.0)
        self.pulse = (self.time * 3.0).sin() * 0.5 + 0.5;

        // Smooth scroll to target positions (cubic ease-out)
        let target_category = self.selected_category as f32;
        let target_item = self.selected_item as f32;

        self.category_scroll = Self::ease_towards(self.category_scroll, target_category, dt * 8.0);
        self.item_scroll = Self::ease_towards(self.item_scroll, target_item, dt * 10.0);

        // Update status message timer
        if self.status_timer > 0.0 {
            self.status_timer -= dt;
            if self.status_timer <= 0.0 {
                self.status_message = None;
            }
        }

        // Update background particles
        self.update_bg_particles(dt);

        // Decay navigation impulse
        self.nav_impulse_x *= 0.95_f32.powf(dt * 60.0);
        self.nav_impulse_y *= 0.95_f32.powf(dt * 60.0);
    }

    /// Update background particle positions
    fn update_bg_particles(&mut self, dt: f32) {
        for particle in &mut self.bg_particles {
            // Update angle for orbital motion
            particle.angle += particle.angular_vel * dt;

            // Base drift velocity
            let base_vx = particle.vx;
            let base_vy = particle.vy;

            // Add navigation impulse influence (particles react to selection changes)
            let impulse_influence = 0.3;
            let total_vx = base_vx + self.nav_impulse_x * impulse_influence * particle.size;
            let total_vy = base_vy + self.nav_impulse_y * impulse_influence * particle.size;

            // Update position with drift
            particle.x += total_vx * dt;
            particle.y += total_vy * dt;

            // Wrap around screen edges (with some margin for orbit)
            let margin = 0.1;
            if particle.x < -margin {
                particle.x += 1.0 + margin * 2.0;
            } else if particle.x > 1.0 + margin {
                particle.x -= 1.0 + margin * 2.0;
            }
            if particle.y < -margin {
                particle.y += 1.0 + margin * 2.0;
            } else if particle.y > 1.0 + margin {
                particle.y -= 1.0 + margin * 2.0;
            }
        }
    }

    /// Smooth easing function
    fn ease_towards(current: f32, target: f32, speed: f32) -> f32 {
        current + (target - current) * speed.min(1.0)
    }

    /// Move selection left (previous category)
    pub fn move_left(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
            self.selected_item = 0; // Reset to first item in new category
            self.nav_impulse_x = -0.5; // Push particles right when moving left
        }
    }

    /// Move selection right (next category)
    pub fn move_right(&mut self) {
        if self.selected_category < self.categories.len().saturating_sub(1) {
            self.selected_category += 1;
            self.selected_item = 0; // Reset to first item in new category
            self.nav_impulse_x = 0.5; // Push particles left when moving right
        }
    }

    /// Move selection up (previous item)
    pub fn move_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.nav_impulse_y = -0.3; // Push particles down when moving up
        }
    }

    /// Move selection down (next item)
    pub fn move_down(&mut self) {
        let current_category = &self.categories[self.selected_category];
        if self.selected_item < current_category.items.len().saturating_sub(1) {
            self.selected_item += 1;
            self.nav_impulse_y = 0.3; // Push particles up when moving down
        }
    }

    /// Get the action of the currently selected item
    pub fn get_selected_action(&self) -> XMBAction {
        if let Some(category) = self.categories.get(self.selected_category) {
            if let Some(item) = category.items.get(self.selected_item) {
                return item.action.clone();
            }
        }
        XMBAction::None
    }

    /// Get the currently selected item's description
    pub fn get_selected_description(&self) -> Option<&str> {
        self.categories
            .get(self.selected_category)
            .and_then(|cat| cat.items.get(self.selected_item))
            .and_then(|item| item.description.as_deref())
    }

    /// Get the currently selected category
    pub fn get_selected_category(&self) -> Option<&XMBCategory> {
        self.categories.get(self.selected_category)
    }

    /// Get the currently selected item
    pub fn get_selected_item(&self) -> Option<&super::menu::XMBItem> {
        self.categories
            .get(self.selected_category)
            .and_then(|cat| cat.items.get(self.selected_item))
    }
}

impl Default for XMBState {
    fn default() -> Self {
        Self::new()
    }
}
