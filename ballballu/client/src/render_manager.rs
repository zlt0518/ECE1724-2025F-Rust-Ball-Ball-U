use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use shared::GameSnapshot;
use shared::objects::PlayerSpec;
use std::io::{self, stdout};

pub struct RenderManager {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    world_width: f32,
    world_height: f32,
}

impl RenderManager {
    pub fn new(world_width: f32, world_height: f32) -> io::Result<Self> {
        //println!("[DEBUG] RenderManager::new called with world size: {}x{}", world_width, world_height);
        enable_raw_mode()?;
        //println!("[DEBUG] Raw mode enabled");
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        //println!("[DEBUG] Entered alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        //println!("[DEBUG] Terminal initialized successfully");

        Ok(Self {
            terminal,
            world_width,
            world_height,
        })
    }

    pub fn render(&mut self, snapshot: &GameSnapshot) -> io::Result<()> {
        let world_width = self.world_width;
        let world_height = self.world_height;
        self.terminal.draw(|f| {
            Self::draw_game_static(f, snapshot, world_width, world_height);
        })?;
        //println!("[DEBUG] RenderManager::render completed successfully");
        Ok(())
    }

    fn draw_game_static(f: &mut Frame, snapshot: &GameSnapshot, world_width: f32, world_height: f32) {
        let size = f.size();

        // Create layout: game area in center, info on sides
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10), // Left margin
                Constraint::Percentage(80), // Game area
                Constraint::Percentage(10), // Right margin
            ])
            .split(size);

        let game_area = chunks[1];

        // Split game area into game canvas and info panel
        let vertical_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(85), // Game canvas
                Constraint::Percentage(15), // Info panel
            ])
            .split(game_area);

        let canvas = vertical_chunks[0];
        let info_panel = vertical_chunks[1];

        // Draw game canvas with borders
        let canvas_block = Block::default()
            .borders(Borders::ALL)
            .title("Game World")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(canvas_block, canvas);

        // Draw players and dots inside the canvas
        let inner_canvas = canvas.inner(Margin::new(1, 1));
        Self::draw_objects_static(f, snapshot, inner_canvas, world_width, world_height);

        // Draw info panel
        Self::draw_info_panel_static(f, snapshot, info_panel);
    }

    fn draw_objects_static(f: &mut Frame, snapshot: &GameSnapshot, area: Rect, world_width: f32, world_height: f32) {
        let canvas_width = area.width as f32;
        let canvas_height = area.height as f32;
        let buffer = f.buffer_mut();

        // Draw dots
        for dot in &snapshot.dots {
            let (x, y) = Self::world_to_screen_static(
                dot.x,
                dot.y,
                world_width,
                world_height,
                canvas_width,
                canvas_height,
            );

            if x >= 0 && x < canvas_width as i32 && y >= 0 && y < canvas_height as i32 {
                let radius = (dot.radius * canvas_width / world_width).max(1.0) as u16;
                let radius = radius.min((canvas_width.min(canvas_height) / 2.0) as u16);

                // Draw dot as a simple character
                if radius <= 1 {
                    let cell = buffer.get_mut(x as u16 + area.x, y as u16 + area.y);
                    cell.set_char('·');
                    cell.set_fg(Color::Rgb(dot.color.0, dot.color.1, dot.color.2));
                } else {
                    // Draw larger dots with a circle approximation
                    for dy in -(radius as i32)..=(radius as i32) {
                        for dx in -(radius as i32)..=(radius as i32) {
                            if dx * dx + dy * dy <= (radius as i32) * (radius as i32) {
                                let px = x + dx;
                                let py = y + dy;
                                if px >= 0
                                    && px < canvas_width as i32
                                    && py >= 0
                                    && py < canvas_height as i32
                                {
                                    let cell = buffer.get_mut(px as u16 + area.x, py as u16 + area.y);
                                    cell.set_char('●');
                                    cell.set_fg(Color::Rgb(dot.color.0, dot.color.1, dot.color.2));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Draw players
        // Use one-tick prediction (based on server tick interval) to smooth movement
        let prediction_ms = snapshot.constants.tick_interval_ms as f32;
        for player in &snapshot.players {
            Self::draw_player_static(
                buffer,
                player,
                area,
                world_width,
                world_height,
                canvas_width,
                canvas_height,
                prediction_ms,
            );
        }
    }

    fn draw_info_panel_static(f: &mut Frame, snapshot: &GameSnapshot, area: Rect) {
        let mut info_lines = vec![
            Line::from(vec![
                Span::styled("Tick: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    snapshot.tick.to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Players: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    snapshot.players.len().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Dots: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    snapshot.dots.len().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];

        // Add player scores
        for player in &snapshot.players {
            info_lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", player.name),
                    Style::default().fg(Self::get_player_color_static(player.id)),
                ),
                Span::styled(
                    format!("Score {} | Size {:.1}", player.score, player.radius),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        let info_block = Block::default()
            .borders(Borders::ALL)
            .title("Game Info")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

        let info_paragraph = Paragraph::new(info_lines)
            .block(info_block)
            .alignment(Alignment::Left);

        f.render_widget(info_paragraph, area);
    }

    fn world_to_screen_static(
        world_x: f32,
        world_y: f32,
        world_w: f32,
        world_h: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> (i32, i32) {
        let x = (world_x / world_w * screen_w).round() as i32;
        let y = (world_y / world_h * screen_h).round() as i32;
        (x, y)
    }

    /// Draw a single player on the screen
    fn draw_player_static(
        buffer: &mut ratatui::buffer::Buffer,
        player: &shared::objects::PlayerSpec,
        area: Rect,
        world_width: f32,
        world_height: f32,
        canvas_width: f32,
        canvas_height: f32,
        prediction_ms: f32,
    ) {
        // Apply a small prediction based on velocity to smooth between snapshots
        let pred_x = player.x + player.vx * (prediction_ms / 1000.0);
        let pred_y = player.y + player.vy * (prediction_ms / 1000.0);

        let (x, y) = Self::world_to_screen_static(
            pred_x,
            pred_y,
            world_width,
            world_height,
            canvas_width,
            canvas_height,
        );

        if x < 0 || x >= canvas_width as i32 || y < 0 || y >= canvas_height as i32 {
            return;
        }

        // Calculate screen radius from world radius (keep as f32 for precision)
        let screen_radius = player.radius * canvas_width / world_width;
        
        // Clamp screen radius to reasonable bounds
        let screen_radius = screen_radius.max(0.5).min(canvas_width.min(canvas_height) / 2.0);

        let player_color = Self::get_player_color_static(player.id);
        
        // terminal cell is ~2x taller than wide
        let char_aspect_ratio: f32 = 2.0;

        // Use separate radii for x and y to produce an ellipse that looks like a circle
        let radius_x = screen_radius;
        let radius_y = screen_radius * char_aspect_ratio;

        // Use f32 for precise circle/ellipse calculation
        let radius_x_i32 = radius_x.ceil() as i32;
        let radius_y_i32 = radius_y.ceil() as i32;

        // For very small radius, just draw center point
        if screen_radius <= 0.5 {
            let cell = buffer.get_mut(x as u16 + area.x, y as u16 + area.y);
            cell.set_char('█');
            cell.set_fg(player_color);
        } else {
            // Draw filled circle using multi-cell approach: dx*dx + dy*dy <= r*r
            // For each terminal cell, check if it's within the circle radius
            let radius_sq = screen_radius * screen_radius;

            for dy in -radius_x_i32..=radius_x_i32 {
                for dx in -radius_x_i32..=radius_x_i32 {
                    // Calculate distance from center for this cell
                    let dx_f = dx as f32;
                    let dy_f = dy as f32 * char_aspect_ratio;
                    // Check if this cell is within the circle
                    if dx_f * dx_f + dy_f * dy_f <= radius_sq {
                        let px = x + dx;
                        let py = y + dy;
                        if px >= 0
                            && px < canvas_width as i32
                            && py >= 0
                            && py < canvas_height as i32
                        {
                            let cell = buffer.get_mut(px as u16 + area.x, py as u16 + area.y);
                            // Use block character instead of background color
                            cell.set_char('█');
                            cell.set_fg(player_color);
                        }
                    }
                }
            }
        }

        // Draw player name above the circle
        if y > 0 && !player.name.is_empty() {
            let name_offset = if screen_radius <= 0.5 { 
                1 
            } else { 
                screen_radius.ceil() as i32 + 1 
            };
            let name_y = (y - name_offset).max(0) as u16;
            if name_y < canvas_height as u16 {
                let name_text = if player.name.len() > 10 {
                    &player.name[..10]
                } else {
                    &player.name
                };
                let name_x = (x - name_text.len() as i32 / 2).max(0) as u16;
                if name_x + name_text.len() as u16 <= canvas_width as u16 {
                    // Draw name directly to buffer
                    for (i, ch) in name_text.chars().enumerate() {
                        if (name_x as usize + i) < canvas_width as usize {
                            let cell = buffer.get_mut(name_x + area.x + i as u16, name_y + area.y);
                            cell.set_char(ch);
                            cell.set_fg(player_color);
                        }
                    }
                }
            }
        }
    }

    fn get_player_color_static(player_id: u64) -> Color {
        // Generate a color based on player ID
        let colors = [
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::White,
        ];
        colors[(player_id as usize) % colors.len()]
    }

    pub fn cell_distance(&mut self, dx: f32, dy: f32) -> f32 {
        // Get the current terminal frame size
        let size = self
            .terminal
            .size()
            .expect("Failed to read terminal size");

        // Recompute the same canvas layout used in draw()
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(size);

        let game_area = chunks[1];

        let vertical_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(85),
                Constraint::Percentage(15),
            ])
            .split(game_area);

        let canvas = vertical_chunks[0];
        let canvas_w = canvas.width as f32;
        let canvas_h = canvas.height as f32;

        // 1 terminal cell worth of world-units
        if dx.abs() > 0.0 {
            self.world_width / canvas_w // horizontal keypress
        } else {
            self.world_height / canvas_h // vertical keypress
        }
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

fn world_units_per_screen_cell(
    world_w: f32,
    world_h: f32,
    canvas_w: f32,
    canvas_h: f32,
    dx: f32,
    dy: f32,
) -> f32 {
    if dx.abs() > 0.0 {
        world_w / canvas_w    // horizontal: 1 cell worth of world-units
    } else {
        world_h / canvas_h    // vertical: 1 cell worth of world-units
    }
}
