use macroquad::prelude::*;
use shared::{GameSnapshot, mechanics};
use std::time::Instant;

pub struct RenderManager {
    world_width: f32,
    world_height: f32,
    camera_x: f32,
    camera_y: f32,
}

impl RenderManager {
    pub fn new(world_width: f32, world_height: f32) -> Self {
        Self {
            world_width,
            world_height,
            camera_x: world_width / 2.0,
            camera_y: world_height / 2.0,
        }
    }

    pub fn render(&mut self, snapshot: &GameSnapshot, received_at: Instant, player_id: Option<u64>) {
        // Clear screen with dark background
        clear_background(Color::from_rgba(10, 10, 15, 255));

        // Get screen dimensions
        let screen_width = screen_width();
        let screen_height = screen_height();

        // Update camera to follow the local player (by player_id)
        // If player_id is set, find the player with that ID, otherwise follow first player
        let player_to_follow = if let Some(id) = player_id {
            snapshot.players.iter().find(|p| p.id == id)
        } else {
            snapshot.players.first()
        };
        
        if let Some(player) = player_to_follow {
            // Use prediction for smooth camera movement
            let pred_seconds = received_at.elapsed().as_secs_f32();
            self.camera_x = player.x + player.vx * pred_seconds;
            self.camera_y = player.y + player.vy * pred_seconds;
        }

        // Calculate viewport bounds (world coordinates visible on screen)
        let viewport_width = 1000.0; // Adjust for zoom level
        let viewport_height = 750.0;
        let min_x = self.camera_x - viewport_width / 2.0;
        let max_x = self.camera_x + viewport_width / 2.0;
        let min_y = self.camera_y - viewport_height / 2.0;
        let max_y = self.camera_y + viewport_height / 2.0;

        // Draw grid for reference
        self.draw_grid(min_x, max_x, min_y, max_y, screen_width, screen_height);

        // Draw world boundaries
        self.draw_world_bounds(min_x, max_x, min_y, max_y, screen_width, screen_height);

        // Draw dots
        for dot in &snapshot.dots {
            // Check if dot is in viewport
            if dot.x >= min_x - dot.radius
                && dot.x <= max_x + dot.radius
                && dot.y >= min_y - dot.radius
                && dot.y <= max_y + dot.radius
            {
                let (screen_x, screen_y) = self.world_to_screen(
                    dot.x,
                    dot.y,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    screen_width,
                    screen_height,
                );
                let screen_radius = self.world_to_screen_size(dot.radius, viewport_width, screen_width);

                draw_circle(
                    screen_x,
                    screen_y,
                    screen_radius.max(2.0),
                    Color::from_rgba(dot.color.0, dot.color.1, dot.color.2, 255),
                );
            }
        }

        // Draw players
        for player in &snapshot.players {
            // Apply client-side prediction for smooth movement
            let pred_seconds = received_at.elapsed().as_secs_f32();
            let predicted_x = player.x + player.vx * pred_seconds;
            let predicted_y = player.y + player.vy * pred_seconds;

            // Check if player is in viewport
            if predicted_x >= min_x - player.radius
                && predicted_x <= max_x + player.radius
                && predicted_y >= min_y - player.radius
                && predicted_y <= max_y + player.radius
            {
                let (screen_x, screen_y) = self.world_to_screen(
                    predicted_x,
                    predicted_y,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    screen_width,
                    screen_height,
                );
                let screen_radius = self.world_to_screen_size(player.radius, viewport_width, screen_width);

                let player_color = Self::get_player_color(player.id);

                // Draw player circle (filled)
                draw_circle(screen_x, screen_y, screen_radius.max(5.0), player_color);

                // Draw player outline
                draw_circle_lines(
                    screen_x,
                    screen_y,
                    screen_radius.max(5.0),
                    2.0,
                    Color::from_rgba(255, 255, 255, 120),
                );

                // Draw player name above the circle
                let name_y = screen_y - screen_radius - 15.0;
                let text_size = 20.0;
                let text_dims = measure_text(&player.name, None, text_size as u16, 1.0);
                let name_x = screen_x - text_dims.width / 2.0;

                // Draw name background
                draw_rectangle(
                    name_x - 4.0,
                    name_y - text_dims.height + 2.0,
                    text_dims.width + 8.0,
                    text_dims.height + 4.0,
                    Color::from_rgba(0, 0, 0, 180),
                );

                // Draw name text
                draw_text(&player.name, name_x, name_y, text_size, WHITE);

                // Draw score below name
                let score_text = format!("Score: {}", player.score);
                let score_dims = measure_text(&score_text, None, 16, 1.0);
                let score_x = screen_x - score_dims.width / 2.0;
                let score_y = name_y + 18.0;
                draw_text(&score_text, score_x, score_y, 16.0, Color::from_rgba(200, 200, 200, 255));
            }
        }

        // Draw UI overlay
        self.draw_ui_overlay(snapshot);
    }

    fn world_to_screen(
        &self,
        world_x: f32,
        world_y: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> (f32, f32) {
        let viewport_w = max_x - min_x;
        let viewport_h = max_y - min_y;
        let x = (world_x - min_x) / viewport_w * screen_w;
        let y = (world_y - min_y) / viewport_h * screen_h;
        (x, y)
    }

    fn world_to_screen_size(&self, world_size: f32, viewport_width: f32, screen_width: f32) -> f32 {
        world_size / viewport_width * screen_width
    }

    fn draw_grid(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        let grid_size = 100.0; // World units
        let viewport_width = max_x - min_x;
        let viewport_height = max_y - min_y;

        // Vertical lines
        let start_x = (min_x / grid_size).floor() * grid_size;
        let mut x = start_x;
        while x <= max_x {
            if x >= 0.0 && x <= self.world_width {
                let screen_x = (x - min_x) / viewport_width * screen_width;
                draw_line(
                    screen_x,
                    0.0,
                    screen_x,
                    screen_height,
                    1.0,
                    Color::from_rgba(30, 30, 40, 255),
                );
            }
            x += grid_size;
        }

        // Horizontal lines
        let start_y = (min_y / grid_size).floor() * grid_size;
        let mut y = start_y;
        while y <= max_y {
            if y >= 0.0 && y <= self.world_height {
                let screen_y = (y - min_y) / viewport_height * screen_height;
                draw_line(
                    0.0,
                    screen_y,
                    screen_width,
                    screen_y,
                    1.0,
                    Color::from_rgba(30, 30, 40, 255),
                );
            }
            y += grid_size;
        }
    }

    fn draw_world_bounds(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        let viewport_width = max_x - min_x;
        let viewport_height = max_y - min_y;

        // Draw world boundaries if visible
        let bounds_color = Color::from_rgba(255, 100, 100, 150);
        let bounds_thickness = 3.0;

        // Left boundary (x = 0)
        if 0.0 >= min_x && 0.0 <= max_x {
            let screen_x = (0.0 - min_x) / viewport_width * screen_width;
            draw_line(screen_x, 0.0, screen_x, screen_height, bounds_thickness, bounds_color);
        }

        // Right boundary (x = world_width)
        if self.world_width >= min_x && self.world_width <= max_x {
            let screen_x = (self.world_width - min_x) / viewport_width * screen_width;
            draw_line(screen_x, 0.0, screen_x, screen_height, bounds_thickness, bounds_color);
        }

        // Top boundary (y = 0)
        if 0.0 >= min_y && 0.0 <= max_y {
            let screen_y = (0.0 - min_y) / viewport_height * screen_height;
            draw_line(0.0, screen_y, screen_width, screen_y, bounds_thickness, bounds_color);
        }

        // Bottom boundary (y = world_height)
        if self.world_height >= min_y && self.world_height <= max_y {
            let screen_y = (self.world_height - min_y) / viewport_height * screen_height;
            draw_line(0.0, screen_y, screen_width, screen_y, bounds_thickness, bounds_color);
        }
    }

    fn draw_ui_overlay(&self, snapshot: &GameSnapshot) {
        let padding = 10.0;
        let line_height = 25.0;
        let mut y = padding;

        // Calculate panel height
        let panel_height = 120.0 + snapshot.players.len() as f32 * line_height;

        // Draw semi-transparent background
        draw_rectangle(0.0, 0.0, 320.0, panel_height, Color::from_rgba(0, 0, 0, 180));

        // Draw game info
        draw_text(
            &format!("Tick: {}", snapshot.tick),
            padding,
            y + 20.0,
            20.0,
            YELLOW,
        );
        y += line_height;

        draw_text(
            &format!("Players: {}", snapshot.players.len()),
            padding,
            y + 20.0,
            20.0,
            YELLOW,
        );
        y += line_height;

        draw_text(
            &format!("Dots: {}", snapshot.dots.len()),
            padding,
            y + 20.0,
            20.0,
            YELLOW,
        );
        y += line_height;

        // Draw separator
        draw_line(
            padding,
            y + 10.0,
            300.0,
            y + 10.0,
            1.0,
            Color::from_rgba(100, 100, 100, 255),
        );
        y += 20.0;

        // Draw player scores (sorted by score)
        let mut players = snapshot.players.clone();
        players.sort_by(|a, b| b.score.cmp(&a.score));

        for player in &players {
            let color = Self::get_player_color(player.id);
            
            // Calculate expected speed based on mechanics
            let expected_speed = mechanics::calculate_speed_from_score(
                player.score,
                snapshot.constants.move_speed_base,
            );
            
            draw_text(
                &format!(
                    "{}: S:{} R:{:.1} Spd:{:.0}",
                    player.name, player.score, player.radius, expected_speed
                ),
                padding,
                y + 20.0,
                18.0,
                color,
            );
            y += line_height;
        }

        // Draw controls hint at bottom
        let controls_y = screen_height() - 100.0;
        draw_rectangle(
            0.0,
            controls_y,
            320.0,
            100.0,
            Color::from_rgba(0, 0, 0, 180),
        );
        
        y = controls_y + 10.0;
        draw_text("Controls:", padding, y + 20.0, 18.0, Color::from_rgba(150, 150, 150, 255));
        y += 25.0;
        draw_text("WASD / Arrow Keys - Move", padding, y + 20.0, 16.0, WHITE);
        y += 20.0;
        draw_text("ESC - Quit", padding, y + 20.0, 16.0, WHITE);
    }

    fn get_player_color(player_id: u64) -> Color {
        let colors = [
            Color::from_rgba(255, 100, 100, 255), // Red
            Color::from_rgba(100, 150, 255, 255), // Blue
            Color::from_rgba(100, 255, 100, 255), // Green
            Color::from_rgba(255, 255, 100, 255), // Yellow
            Color::from_rgba(255, 100, 255, 255), // Magenta
            Color::from_rgba(100, 255, 255, 255), // Cyan
            Color::from_rgba(255, 200, 100, 255), // Orange
        ];
        colors[(player_id as usize) % colors.len()]
    }
}
