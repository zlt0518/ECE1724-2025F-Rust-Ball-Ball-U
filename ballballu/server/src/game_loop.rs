use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::websocket_manager::WebSocketManager;
use crate::game_state::GameState;
use shared::mechanics::{update_position, dot_collision_check, cells_collisions_check};

pub struct GameLoop {
    pub ws: Arc<WebSocketManager>,
}

impl GameLoop {
    pub fn new(ws: Arc<WebSocketManager>) -> Self {
        Self { ws }
    }

    pub async fn run(&self) {
        // Phase 3: Get tick interval from GameState
        let tick_ms = {
            let gs = self.ws.game_state.lock().await;
            gs.constants.tick_interval_ms
        };

        let mut ticker = interval(Duration::from_millis(tick_ms));

        loop {
            ticker.tick().await;

            {
                let mut gs = self.ws.game_state.lock().await;
                
                // Apply pending moves from input commands
                gs.apply_pending_moves();
                
                // Phase 4: Update player positions based on remaining distance
                let player_ids: Vec<u64> = gs.players.keys().cloned().collect();
                let move_speed_base = gs.constants.move_speed_base;
                for id in player_ids {
                    // Then get mutable player reference
                    if let Some(player) = gs.players.get_mut(&id) {
                        // Calculate current speed based on score
                        let current_speed = shared::mechanics::calculate_speed_from_score(
                            player.score,
                            move_speed_base
                        );
                        player.speed = current_speed;
                        
                        // Update position using shared mechanics (new signature)
                        update_position(player, current_speed, tick_ms as f32);
                    }
                }

                // Phase 5: Handle player-dot collisions
                handle_player_dot_collision(&mut gs);

                // Phase 5: Handle player-player collisions
                handle_player_player_collision(&mut gs);

                // Phase 3: Increment tick
                gs.tick += 1;
            }

            // Phase 3: Broadcast snapshot every tick
            println!("[DEBUG] GameLoop: About to broadcast state (tick: {})", {
                let gs = self.ws.game_state.lock().await;
                gs.tick
            });
            self.ws.broadcast_state().await;
            //println!("[DEBUG] GameLoop: Broadcast completed");
        }
    }
}

// Phase 5: Player vs Dot collision handler
fn handle_player_dot_collision(gs: &mut GameState) {
    let mut eaten = Vec::new();
    
    for (pid, p) in gs.players.iter() {
        for (did, d) in gs.dots.iter() {
            if dot_collision_check(p, d) {
                eaten.push((*pid, *did));
            }
        }
    }

    // Apply effects: remove dots and increase player score/radius
    for (pid, did) in eaten {
        if gs.dots.remove(&did).is_some() {
            if let Some(player) = gs.players.get_mut(&pid) {
                // Increase score
                player.score += 1;
                // Recalculate radius based on score
                player.radius = shared::mechanics::calculate_radius_from_score(
                    player.score,
                    10.0 // base radius
                );
                println!("Player {} ate Dot {}", pid, did);
            }
        }
    }
}

// Phase 5: Player vs Player collision handler
fn handle_player_player_collision(gs: &mut GameState) {
    let ids: Vec<u64> = gs.players.keys().cloned().collect();
    let mut to_remove = Vec::new();

    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            let id_a = ids[i];
            let id_b = ids[j];

            if let (Some(a), Some(b)) = (gs.players.get(&id_a), gs.players.get(&id_b)) {
                if cells_collisions_check(a, b) {
                    // Check if one player can eat the other
                    let size_threshold = gs.constants.collide_size_fraction;
                    
                    if a.radius > b.radius * size_threshold {
                        // A can eat B
                        println!("Player {} ate Player {}", id_a, id_b);
                        to_remove.push((id_a, id_b));
                    } else if b.radius > a.radius * size_threshold {
                        // B can eat A
                        println!("Player {} ate Player {}", id_b, id_a);
                        to_remove.push((id_b, id_a));
                    }
                }
            }
        }
    }

    // Apply consumption effects
    for (eater_id, eaten_id) in to_remove {
        if let Some(eaten) = gs.players.get(&eaten_id) {
            let eaten_score = eaten.score;
            
            // Remove eaten player (this also removes input)
            gs.remove_player(eaten_id);
            
            // Update eater
            if let Some(eater) = gs.players.get_mut(&eater_id) {
                eater.score += eaten_score;
                // Recalculate radius from score
                eater.radius = shared::mechanics::calculate_radius_from_score(
                    eater.score,
                    10.0 // base radius
                );
            }
        }
    }
}
