use macroquad::prelude::*;
use shared::{GameSnapshot, GameStatus};
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

    pub fn render(&mut self, snapshot: &GameSnapshot, received_at: Instant, player_id: Option<u64>, client_ready: bool, show_name_input: bool, player_name: &str, join_time: Option<Instant>) {
        // Clear screen with dark background
        clear_background(Color::from_rgba(10, 10, 15, 255));

        // Get screen dimensions
        let screen_width = screen_width();
        let screen_height = screen_height();

        // Check game status - if client hasn't pressed space yet, show start page even if server says Playing
        match snapshot.status {
            GameStatus::WaitingToStart => {
                // Display start page
                self.draw_start_page(screen_width, screen_height, show_name_input, player_name);
            }
            GameStatus::Playing => {
                // If client hasn't pressed space yet, still show the start page
                if !client_ready {
                    self.draw_start_page(screen_width, screen_height, show_name_input, player_name);
                } else {
                    // Gameplay rendering
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

                            // Draw player name and score stacked above the circle (avoid overlap as radius grows)
                            let display_name = if player.name.trim().is_empty() {
                                format!("Player {}", player.id)
                            } else {
                                player.name.clone()
                            };

                            // Text sizes
                            let name_text_size = 20u16;
                            let score_text_size = 16u16;

                            // Prepare texts and dimensions
                            let name_dims = measure_text(&display_name, None, name_text_size, 1.0);
                            let score_text = format!("Score: {}", player.score);
                            let score_dims = measure_text(&score_text, None, score_text_size, 1.0);

                            // Spacing and padding (pixels)
                            let padding_between_circle_and_stack = 8.0; // gap from circle top to stacked texts
                            let inter_text_spacing = 10.0; // spacing between name and score

                            // Total stacked height (approx) and top Y of the stack
                            let stack_height = name_dims.height + inter_text_spacing + score_dims.height;
                            let stack_top = screen_y - screen_radius - padding_between_circle_and_stack - stack_height;

                            // Compute baseline positions consistent with how measure_text and draw_text are used.
                            // The previous code used "name_y - text_dims.height + 2.0" for rectangle top, so we keep a small offset of 2.0 to match visuals.
                            let name_y = stack_top + name_dims.height - 2.0;
                            let name_x = screen_x - name_dims.width / 2.0;

                            // Draw name background
                            draw_rectangle(
                                name_x - 4.0,
                                name_y - name_dims.height + 2.0,
                                name_dims.width + 8.0,
                                name_dims.height + 4.0,
                                Color::from_rgba(0, 0, 0, 180),
                            );

                            // Draw name text
                            draw_text(&display_name, name_x, name_y, name_text_size as f32, WHITE);

                            // Score baseline: placed below name with configured spacing
                            let score_x = screen_x - score_dims.width / 2.0;
                            let score_y = stack_top + name_dims.height + inter_text_spacing + score_dims.height - 2.0;
                            draw_text(&score_text, score_x, score_y, score_text_size as f32, Color::from_rgba(200, 200, 200, 255));
                        }
                    }

                    // Draw UI overlay
                    self.draw_ui_overlay(snapshot,player_id, join_time);

                    // Show controls panel only for first 3 seconds after joining
                    if let Some(t) = join_time {
                        if t.elapsed().as_secs_f32() < 3.0 {
                            self.draw_controls_panel();
                        }
                    }
                }
            }
            // May not be used for argio style game
            // GameStatus::GameOver => {
            //     // Display game over screen
            //     self.draw_game_over_page(screen_width, screen_height);
            // }
        }
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

    fn draw_ui_overlay(&self, snapshot: &GameSnapshot, player_id: Option<u64>, join_time: Option<Instant>) {
        let padding = 10.0;
        let line = 25.0;
        let mut y = padding;

        // local player score
        let local_score = player_id
            .and_then(|id| snapshot.players.iter().find(|p| p.id == id))
            .map(|p| p.score)
            .unwrap_or(0);

        // top 3 leaderboard
        let mut players = snapshot.players.clone();
        players.sort_by(|a, b| b.score.cmp(&a.score));
        let top3 = players.into_iter().take(3).collect::<Vec<_>>();

        // panel height
        let panel_h = 120.0 + top3.len() as f32 * line;

        // background box
        draw_rectangle(0.0, 0.0, 260.0, panel_h, Color::from_rgba(0, 0, 0, 180));

        // time since join
        let elapsed_ms = join_time.map(|t| t.elapsed().as_millis() as u32).unwrap_or(0);
        let secs = elapsed_ms / 1000;
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;

        draw_text(
            &format!("Time: {:02}:{:02}:{:02}", h, m, s),
            padding,
            y + 20.0,
            20.0,
            YELLOW,
        );
        y += line;

        // local score
        draw_text(
            &format!("Score: {}", local_score),
            padding,
            y + 20.0,
            20.0,
            YELLOW,
        );
        y += line;

        // separator
        draw_line(padding, y + 10.0, 240.0, y + 10.0, 1.0, Color::from_rgba(120, 120, 120, 255));
        y += 20.0;

        draw_text("Top Players:", padding, y + 20.0, 18.0, WHITE);
        y += line;

        // top 3 entries
        for (i, p) in top3.iter().enumerate() {
            let color = Self::get_player_color(p.id);

            let name = if p.name.is_empty() { "Anonymous" } else { p.name.as_str() };

            draw_text(
                &format!("{}. {} (S:{})", i + 1, name, p.score),
                padding,
                y + 20.0,
                18.0,
                color,
            );
            y += line;
        }
    }

    fn draw_controls_panel(&self) {
        let w = 260.0;
        let h = 90.0;
        let x = 10.0;
        let y = screen_height() - h - 10.0;

        // Background
        draw_rectangle(x, y, w, h, Color::from_rgba(0, 0, 0, 180));

        // Title
        draw_text(
            "Controls:",
            x + 10.0,
            y + 28.0,
            20.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        // WASD
        draw_text(
            "WASD / Arrow Keys - Move",
            x + 10.0,
            y + 50.0,
            16.0,
            WHITE,
        );

        // Quit
        draw_text(
            "ESC - Quit",
            x + 10.0,
            y + 70.0,
            16.0,
            WHITE,
        );
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

    fn draw_start_page(&self, screen_width: f32, screen_height: f32, show_name_input: bool, player_name: &str) {
        // Draw semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::from_rgba(0, 0, 0, 200));

        // Draw title
        let title = "ECE1724::RUST::BALL BALL U";
        let title_size = 80.0;
        let title_dims = measure_text(title, None, title_size as u16, 1.0);
        let title_x = screen_width / 2.0 - title_dims.width / 2.0;
        let title_y = screen_height / 2.0 - 120.0;
        draw_text(title, title_x, title_y, title_size, Color::from_rgba(100, 255, 100, 255));

        // Draw subtitle
        let subtitle = "University of Toronto";
        let subtitle_size = 30.0;
        let subtitle_dims = measure_text(subtitle, None, subtitle_size as u16, 1.0);
        let subtitle_x = screen_width / 2.0 - subtitle_dims.width / 2.0;
        let subtitle_y = title_y + 80.0;
        draw_text(subtitle, subtitle_x, subtitle_y, subtitle_size, Color::from_rgba(150, 150, 255, 255));

        // Draw author information
        let authors = vec![
            "Litao(John) Zhou - 1006013092",
            "Siyu Shao - 1007147204",
            "Chuyue Zhang - 1005728303",
        ];
        let author_size = 20.0;
        let author_y_start = subtitle_y + 60.0;
        
        for (i, author) in authors.iter().enumerate() {
            let author_dims = measure_text(author, None, author_size as u16, 1.0);
            let author_x = screen_width / 2.0 - author_dims.width / 2.0;
            let author_y = author_y_start + (i as f32 * 25.0);
            draw_text(author, author_x, author_y, author_size, Color::from_rgba(200, 200, 200, 255));
        }

        // Draw player name input box
        if show_name_input {
            let input_y = author_y_start + (authors.len() as f32 * 25.0) + 40.0;
            
            // Draw label
            let label = "Enter Your Player Name (max 15 characters):";
            let label_size = 20.0;
            let label_dims = measure_text(label, None, label_size as u16, 1.0);
            let label_x = screen_width / 2.0 - label_dims.width / 2.0;
            draw_text(label, label_x, input_y, label_size, Color::from_rgba(200, 200, 200, 255));
            
            // Draw input box background
            let box_width = 400.0;
            let box_height = 40.0;
            let box_x = screen_width / 2.0 - box_width / 2.0;
            let box_y = input_y + 35.0;
            draw_rectangle(box_x, box_y, box_width, box_height, Color::from_rgba(50, 50, 50, 255));
            draw_rectangle_lines(box_x, box_y, box_width, box_height, 2.0, Color::from_rgba(200, 200, 200, 255));
            
            // Draw typed text
            let text_size = 24.0;
            draw_text(player_name, box_x + 10.0, box_y + 28.0, text_size, Color::from_rgba(255, 255, 255, 255));
            
            // Draw character count
            let char_count_text = format!("{}/15", player_name.len());
            let char_count_size = 16.0;
            let char_count_dims = measure_text(&char_count_text, None, char_count_size as u16, 1.0);
            let char_count_x = screen_width / 2.0 - char_count_dims.width / 2.0;
            draw_text(&char_count_text, char_count_x, box_y + box_height + 25.0, char_count_size, Color::from_rgba(150, 150, 150, 255));
        }

        // Instruction text depends on whether name is empty or not
        let instruction = if player_name.is_empty() {
            "Press ENTER to start as Anonymous"
        } else {
            "Press ENTER to continue"
        };

        let instruction_size = 32.0;
        let instruction_dims = measure_text(instruction, None, instruction_size as u16, 1.0);
        let instruction_x = screen_width / 2.0 - instruction_dims.width / 2.0;

        // Place instruction below input box (input box bottom = box_y + box_height)
        let instruction_y = {
            let authors_height = authors.len() as f32 * 25.0;
            let input_y = author_y_start + authors_height + 40.0;
            let box_y = input_y + 35.0;
            box_y + 40.0 + 70.0  // box height + padding
        };

        draw_text(
            instruction,
            instruction_x,
            instruction_y,
            instruction_size,
            Color::from_rgba(255, 200, 100, 255),
        );

        // (Removed controls from start page)
        // Draw controls hint
        // let controls = "Use WASD or Arrow Keys to move\nESC to quit";
        // let controls_size = 20.0;
        // let controls_y = screen_height - 100.0;
        // draw_text(controls, 20.0, controls_y, controls_size, Color::from_rgba(200, 200, 200, 200));
    }

    // May not be used for argio style game
    // fn draw_game_over_page(&self, screen_width: f32, screen_height: f32) {
    //     // Draw semi-transparent overlay
    //     draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::from_rgba(0, 0, 0, 200));

    //     // Draw title
    //     let title = "GAME OVER";
    //     let title_size = 80.0;
    //     let title_dims = measure_text(title, None, title_size as u16, 1.0);
    //     let title_x = screen_width / 2.0 - title_dims.width / 2.0;
    //     let title_y = screen_height / 2.0 - 60.0;
    //     draw_text(title, title_x, title_y, title_size, Color::from_rgba(255, 100, 100, 255));

    //     // Draw restart instruction
    //     let restart = "Press ENTER to Return to Start";
    //     let restart_size = 30.0;
    //     let restart_dims = measure_text(restart, None, restart_size as u16, 1.0);
    //     let restart_x = screen_width / 2.0 - restart_dims.width / 2.0;
    //     let restart_y = screen_height / 2.0 + 60.0;
    //     draw_text(restart, restart_x, restart_y, restart_size, Color::from_rgba(255, 200, 100, 255));
    // }
}
