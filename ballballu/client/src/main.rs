mod render_manager;

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use shared::{GameSnapshot, protocol::ServerMessage};
use serde_json;

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8000";
    
    // Initialize render manager with default world size
    // You may want to get these from the server's initial message
    let world_width = 2000.0;
    let world_height = 2000.0;
    
    let mut render_manager = match render_manager::RenderManager::new(world_width, world_height) {
        Ok(rm) => rm,
        Err(e) => {
            eprintln!("Failed to initialize render manager: {:?}", e);
            return;
        }
    };

    println!("[DEBUG] Initializing render manager...");
    println!("Connecting to {}", url);

    let (ws_stream, _) = match connect_async(url).await {
        Ok(c) => {
            println!("[DEBUG] Successfully connected to server");
            c
        },
        Err(e) => {
            eprintln!("[DEBUG] Failed to connect: {:?}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Channel to send game snapshots from websocket task to main loop
    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel::<GameSnapshot>();

    // Spawn a task to receive messages from the server
    let read_handle = tokio::spawn(async move {
        println!("[DEBUG] WebSocket read task started");
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("[DEBUG] Received text message from server: {} bytes", text.len());
                    // Try to parse as ServerMessage first
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(server_msg) => {
                            println!("[DEBUG] Successfully parsed ServerMessage: {:?}", 
                                match &server_msg {
                                    ServerMessage::Welcome(_) => "Welcome",
                                    ServerMessage::StateUpdate(_) => "StateUpdate",
                                    ServerMessage::Bye(_) => "Bye",
                                }
                            );
                            // Extract GameSnapshot from StateUpdate
                            if let ServerMessage::StateUpdate(state_update) = server_msg {
                                let snapshot = state_update.snapshot;
                                println!("[DEBUG] Extracted snapshot: tick={}, players={}, dots={}", 
                                    snapshot.tick, snapshot.players.len(), snapshot.dots.len());
                                if let Err(_) = snapshot_tx.send(snapshot) {
                                    println!("[DEBUG] Failed to send snapshot to channel");
                                    break;
                                } else {
                                    println!("[DEBUG] Successfully sent snapshot to render channel");
                                }
                            } else {
                                println!("[DEBUG] Received non-StateUpdate message, ignoring");
                            }
                        }
                        Err(e) => {
                            println!("[DEBUG] Failed to parse ServerMessage: {:?}", e);
                            println!("[DEBUG] Raw message (first 200 chars): {}", 
                                text.chars().take(200).collect::<String>());
                            // Try direct GameSnapshot parse as fallback (for debugging)
                            match serde_json::from_str::<GameSnapshot>(&text) {
                                Ok(snapshot) => {
                                    println!("[DEBUG] Fallback: Direct GameSnapshot parse succeeded");
                                    let _ = snapshot_tx.send(snapshot);
                                }
                                Err(e2) => {
                                    println!("[DEBUG] Fallback: Direct GameSnapshot parse also failed: {:?}", e2);
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("[DEBUG] Server closed connection");
                    break;
                }
                Err(e) => {
                    eprintln!("[DEBUG] WebSocket error: {:?}", e);
                    break;
                }
                _ => {
                    println!("[DEBUG] Received non-text message");
                }
            }
        }
        println!("[DEBUG] WebSocket read task ended");
    });

    println!("[DEBUG] Entering main render loop");
    
    // Main loop: render game state and handle input
    loop {
        tokio::select! {
            // Receive game snapshot and render
            snapshot = snapshot_rx.recv() => {
                match snapshot {
                    Some(snap) => {
                        println!("[DEBUG] Received snapshot in main loop: tick={}, players={}, dots={}", 
                            snap.tick, snap.players.len(), snap.dots.len());
                        if let Err(e) = render_manager.render(&snap) {
                            eprintln!("[DEBUG] Render error: {:?}", e);
                            break;
                        } else {
                            println!("[DEBUG] Successfully rendered snapshot");
                        }
                    }
                    None => {
                        println!("[DEBUG] Snapshot channel closed, server disconnected");
                        break;
                    }
                }
            }
            // Check if read task finished
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if read_handle.is_finished() {
                    println!("[DEBUG] Read task finished, exiting main loop");
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = write.send(Message::Close(None)).await;
}
