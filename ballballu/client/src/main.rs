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
            match msg {
                Ok(Message::Text(text)) => {
                    // Try to parse as ServerMessage
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(server_msg) => {
                            match server_msg {
                                ServerMessage::Welcome(welcome) => {
                                    println!("Welcomed! Player ID: {}", welcome.player_id);
                                    // Send player_id to main loop
                                    let _ = player_id_tx.send(welcome.player_id);
                                }
                                ServerMessage::StateUpdate(state_update) => {
                                    let snapshot = state_update.snapshot;
                                    if snapshot_tx
                                        .send(ClientSnapshot { 
                                            snapshot, 
                                            received_at: Instant::now() 
                                        })
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                ServerMessage::Bye(bye) => {
                                    println!("Server says goodbye: {}", bye.reason);
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            // Try direct GameSnapshot parse as fallback
                            if let Ok(snapshot) = serde_json::from_str::<GameSnapshot>(&text) {
                                let _ = snapshot_tx
                                    .send(ClientSnapshot { 
                                        snapshot, 
                                        received_at: Instant::now() 
                                    })
                                    .ok();
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed connection");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {
                    // Ignore other message types
                }
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

    loop {
        // Check for shutdown signal (non-blocking)
        if let Ok(_) = shutdown_rx.try_recv() {
            connection_lost = true;
            should_exit = true;
        }

        // Poll for keyboard input (one-click movement). We pass the local
        // player's radius so the client can compute step distance.
        let player_radius = latest_snapshot
            .as_ref()
            .and_then(|s| s.snapshot.players.first().map(|p| p.radius));
        if input_manager.poll_input(player_radius) {
            should_exit = true;
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
            render_manager.render(&snap.snapshot, snap.received_at, player_id);
            
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
