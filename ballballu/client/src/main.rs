mod render_manager;
mod input_manager;

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use shared::{GameSnapshot, protocol::{ServerMessage, ClientMessage}};
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

    //println!("[DEBUG] Initializing render manager...");
    println!("Connecting to {}", url);

    let (ws_stream, _) = match connect_async(url).await {
        Ok(c) => {
            //println!("[DEBUG] Successfully connected to server");
            c
        },
        Err(e) => {
            eprintln!("Failed to connect: {:?}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Channel to send game snapshots from websocket task to main loop
    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel::<GameSnapshot>();
    
    // Channel to send input commands to server
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<ClientMessage>();
    
    // Initialize input manager
    let input_manager = input_manager::InputManager::new(input_tx.clone());

    // Spawn a task to receive messages from the server
    let read_handle = tokio::spawn(async move {
        //println!("[DEBUG] WebSocket read task started");
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    //println!("[DEBUG] Received text message from server: {} bytes", text.len());
                    // Try to parse as ServerMessage first
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(server_msg) => {
                            // Extract GameSnapshot from StateUpdate
                            if let ServerMessage::StateUpdate(state_update) = server_msg {
                                let snapshot = state_update.snapshot;
                                if let Err(_) = snapshot_tx.send(snapshot) {
                                    //println!("[DEBUG] Failed to send snapshot to channel");
                                    break;
                                } else {
                                    //println!("[DEBUG] Successfully sent snapshot to render channel");
                                }
                            } else {
                                //println!("[DEBUG] Received non-StateUpdate message, ignoring");
                            }
                        }
                        Err(_e) => {
                            // Try direct GameSnapshot parse as fallback (for debugging)
                            match serde_json::from_str::<GameSnapshot>(&text) {
                                Ok(snapshot) => {
                                    //println!("[DEBUG] Fallback: Direct GameSnapshot parse succeeded");
                                    let _ = snapshot_tx.send(snapshot);
                                }
                                Err(_e2) => {
                                    //println!("[DEBUG] Fallback: Direct GameSnapshot parse also failed: {:?}", e2);
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    //println!("[DEBUG] Server closed connection");
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
    });

    // Spawn a task to send input commands to server
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = input_rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if write.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });
    
    // Main loop: render game state and handle input
    let mut should_exit = false;
    loop {
        // Poll for keyboard input (non-blocking)
        if input_manager.poll_input() {
            should_exit = true;
        }
        
        tokio::select! {
            // Receive game snapshot and render
            snapshot = snapshot_rx.recv() => {
                match snapshot {
                    Some(snap) => {
                        if let Err(e) = render_manager.render(&snap) {
                            eprintln!("Render error: {:?}", e);
                            break;
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
            // Check if tasks finished or we should exit
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                if should_exit || read_handle.is_finished() || write_handle.is_finished() {
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = write_handle.abort();
    let _ = read_handle.abort();
}
