mod render_manager;
mod input_manager;
mod websocket;

use tokio::runtime::Runtime;
use macroquad::prelude::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use shared::{GameSnapshot, protocol::{ServerMessage, ClientMessage}};
use serde_json;
use crate::websocket::ClientSnapshot;
use std::time::Instant;

fn window_conf() -> Conf {
    Conf {
        window_title: "Ball Ball U - Agar.io Clone".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

/// Handle incoming text messages from the server
/// Returns true if the connection should be closed
fn handle_text_message(
    text: &str,
    snapshot_tx: &mpsc::UnboundedSender<ClientSnapshot>,
    player_id_tx: &mpsc::UnboundedSender<u64>,
) -> bool {
    // Try to parse as ServerMessage
    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(text) {
        return handle_server_message(server_msg, snapshot_tx, player_id_tx);
    }
    
    // Try direct GameSnapshot parse as fallback
    if let Ok(snapshot) = serde_json::from_str::<GameSnapshot>(text) {
        let _ = snapshot_tx.send(ClientSnapshot {
            snapshot,
            received_at: Instant::now(),
        });
    }
    
    false
}

/// Handle parsed server messages
/// Returns true if the connection should be closed
fn handle_server_message(
    msg: ServerMessage,
    snapshot_tx: &mpsc::UnboundedSender<ClientSnapshot>,
    player_id_tx: &mpsc::UnboundedSender<u64>,
) -> bool {
    match msg {
        ServerMessage::Welcome(welcome) => {
            println!("Welcomed! Player ID: {}", welcome.player_id);
            let _ = player_id_tx.send(welcome.player_id);
            false
        }
        ServerMessage::StateUpdate(state_update) => {
            let snapshot = state_update.snapshot;
            snapshot_tx
                .send(ClientSnapshot {
                    snapshot,
                    received_at: Instant::now(),
                })
                .is_err()
        }
        ServerMessage::Bye(bye) => {
            println!("Server says goodbye: {}", bye.reason);
            true
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // +++Create tokio runtime manually (macroquad does NOT supply a reactor)
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    let url = "ws://127.0.0.1:8000";
    
    // Initialize render manager with default world size
    // These should match server's world size
    let world_width = 2000.0;
    let world_height = 2000.0;
    let mut render_manager = render_manager::RenderManager::new(world_width, world_height);

    println!("Connecting to {}", url);

    // Show connecting screen
    clear_background(BLACK);
    draw_text(
        "Connecting to server...",
        screen_width() / 2.0 - 150.0,
        screen_height() / 2.0,
        30.0,
        WHITE,
    );
    next_frame().await;

    // Connect to WebSocket server
    // let (ws_stream, _) = match connect_async(url).await {
    let (ws_stream, _) = match rt.block_on(async { connect_async(url).await }) {
        Ok(c) => {
            println!("Successfully connected to server");
            c
        },
        Err(e) => {
            eprintln!("Failed to connect: {:?}", e);
            // Show error on screen
            loop {
                clear_background(BLACK);
                draw_text(
                    "Failed to connect to server!",
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0 - 20.0,
                    30.0,
                    RED,
                );
                draw_text(
                    &format!("Error: {:?}", e),
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0 + 20.0,
                    20.0,
                    Color::from_rgba(255, 150, 150, 255),
                );
                draw_text(
                    "Press ESC to exit",
                    screen_width() / 2.0 - 100.0,
                    screen_height() / 2.0 + 60.0,
                    20.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                next_frame().await;
            }
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Channel to send game snapshots from websocket task to main loop
    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel::<ClientSnapshot>();
    
    // Channel to send input commands to server
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<ClientMessage>();
    
    // Channel to signal shutdown
    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
    
    // Channel to send player_id from websocket task to main loop
    let (player_id_tx, mut player_id_rx) = mpsc::unbounded_channel::<u64>();
    
    // Initialize input manager
    let input_manager = input_manager::InputManager::new(input_tx.clone());

    // Spawn a task to receive messages from the server
    let read_handle = rt.spawn(async move {
        while let Some(msg) = read.next().await {
            let should_break = match msg {
                Ok(Message::Text(text)) => {
                    handle_text_message(&text, &snapshot_tx, &player_id_tx)
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed connection");
                    true
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    true
                }
                _ => false, // Ignore other message types
            };
            
            if should_break {
                break;
            }
        }
        // Notify main loop that server disconnected
        let _ = shutdown_tx.send(());
    });

    // Spawn a task to send input commands to server
    let write_handle = rt.spawn(async move {
        while let Some(msg) = input_rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if write.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });
    
    // Main game loop
    let mut should_exit = false;
    let mut latest_snapshot: Option<ClientSnapshot> = None;
    let mut connection_lost = false;
    let mut frames_without_update = 0;
    let mut player_id: Option<u64> = None;
    let mut client_ready = false;  // Track if this client has pressed enter
    let mut player_name = String::new();  // Player name input
    let mut name_submitted = false;  // Track if name has been submitted

    loop {
        // Check for shutdown signal (non-blocking)
        if let Ok(_) = shutdown_rx.try_recv() {
            connection_lost = true;
            should_exit = true;
        }

        // Handle text input for player name on start screen
        if !name_submitted {
            // Check for backspace
            if is_key_pressed(KeyCode::Backspace) {
                if !player_name.is_empty() {
                    player_name.pop();
                }
            }
            
            // Check for enter
            if is_key_pressed(KeyCode::Enter) {
                name_submitted = true;
                client_ready = true;
                let _ = input_tx.send(ClientMessage::Join { name: player_name.clone() });
                let _ = input_tx.send(ClientMessage::Ready);
            }
            
            // Handle alphanumeric and space input
            if let Some(key) = get_last_key_pressed() {
                if player_name.len() < 15 {
                    match key {
                        KeyCode::A => player_name.push('A'),
                        KeyCode::B => player_name.push('B'),
                        KeyCode::C => player_name.push('C'),
                        KeyCode::D => player_name.push('D'),
                        KeyCode::E => player_name.push('E'),
                        KeyCode::F => player_name.push('F'),
                        KeyCode::G => player_name.push('G'),
                        KeyCode::H => player_name.push('H'),
                        KeyCode::I => player_name.push('I'),
                        KeyCode::J => player_name.push('J'),
                        KeyCode::K => player_name.push('K'),
                        KeyCode::L => player_name.push('L'),
                        KeyCode::M => player_name.push('M'),
                        KeyCode::N => player_name.push('N'),
                        KeyCode::O => player_name.push('O'),
                        KeyCode::P => player_name.push('P'),
                        KeyCode::Q => player_name.push('Q'),
                        KeyCode::R => player_name.push('R'),
                        KeyCode::S => player_name.push('S'),
                        KeyCode::T => player_name.push('T'),
                        KeyCode::U => player_name.push('U'),
                        KeyCode::V => player_name.push('V'),
                        KeyCode::W => player_name.push('W'),
                        KeyCode::X => player_name.push('X'),
                        KeyCode::Y => player_name.push('Y'),
                        KeyCode::Z => player_name.push('Z'),
                        KeyCode::Key0 => player_name.push('0'),
                        KeyCode::Key1 => player_name.push('1'),
                        KeyCode::Key2 => player_name.push('2'),
                        KeyCode::Key3 => player_name.push('3'),
                        KeyCode::Key4 => player_name.push('4'),
                        KeyCode::Key5 => player_name.push('5'),
                        KeyCode::Key6 => player_name.push('6'),
                        KeyCode::Key7 => player_name.push('7'),
                        KeyCode::Key8 => player_name.push('8'),
                        KeyCode::Key9 => player_name.push('9'),
                        KeyCode::Space => player_name.push(' '),
                        KeyCode::Minus => player_name.push('_'),
                        _ => {}
                    }
                }
            }
        } else {
            // Poll for keyboard input (one-click movement). We pass the local
            // player's radius so the client can compute step distance.
            let player_radius = latest_snapshot
                .as_ref()
                .and_then(|s| s.snapshot.players.first().map(|p| p.radius));
            let (should_exit_input, _enter_pressed) = input_manager.poll_input(player_radius);
            if should_exit_input {
                should_exit = true;
            }
        }

        // Try to receive player_id (non-blocking)
        while let Ok(id) = player_id_rx.try_recv() {
            player_id = Some(id);
            println!("Received player_id: {}", id);
        }

        // Try to receive new snapshots (non-blocking, drain all pending)
        let mut received_new_snapshot = false;
        while let Ok(snap) = snapshot_rx.try_recv() {
            latest_snapshot = Some(snap);
            received_new_snapshot = true;
            frames_without_update = 0;
        }

        if !received_new_snapshot {
            frames_without_update += 1;
        }

        // Render the game
        if let Some(ref snap) = latest_snapshot {
            render_manager.render(&snap.snapshot, snap.received_at, player_id, client_ready, !name_submitted, &player_name);
            
            // Show warning if no updates for a while
            if frames_without_update > 120 {
                draw_text(
                    "No updates from server...",
                    screen_width() / 2.0 - 120.0,
                    30.0,
                    20.0,
                    Color::from_rgba(255, 200, 0, 255),
                );
            }
        } else {
            // Show connecting message
            clear_background(BLACK);
            draw_text(
                "Waiting for game state...",
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0,
                30.0,
                WHITE,
            );
        }

        // Show connection lost message overlay
        if connection_lost {
            let box_width = 400.0;
            let box_height = 100.0;
            let box_x = screen_width() / 2.0 - box_width / 2.0;
            let box_y = screen_height() / 2.0 - box_height / 2.0;
            
            draw_rectangle(
                box_x,
                box_y,
                box_width,
                box_height,
                Color::from_rgba(0, 0, 0, 220),
            );
            
            draw_text(
                "Connection Lost",
                screen_width() / 2.0 - 100.0,
                screen_height() / 2.0 - 10.0,
                30.0,
                RED,
            );
            draw_text(
                "Press ESC to exit",
                screen_width() / 2.0 - 80.0,
                screen_height() / 2.0 + 25.0,
                20.0,
                WHITE,
            );
        }

        // Check if tasks finished
        if read_handle.is_finished() || write_handle.is_finished() {
            should_exit = true;
        }

        if should_exit {
            break;
        }

        next_frame().await;
    }

    // Cleanup
    read_handle.abort();
    write_handle.abort();
    
    println!("Client shutting down...");
}
