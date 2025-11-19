use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;

use shared::protocol::{ClientMessage, ServerMessage, StateUpdateMessage};
use crate::game_state::GameState;

pub type Tx = mpsc::UnboundedSender<Message>;
pub type Rx = mpsc::UnboundedReceiver<Message>;

pub struct WebSocketManager {
    pub addr: String,
    pub next_player_id: Arc<Mutex<u64>>,
    pub game_state: Arc<Mutex<GameState>>,
    // Phase 3: Connections for broadcasting
    pub connections: Arc<Mutex<HashMap<u64, Tx>>>,
}

impl WebSocketManager {
    pub async fn new(addr: &str) -> Self {
        let constants = shared::GameConstant {
            tick_interval_ms: 50,
            collide_size_fraction: 1.1,
            move_speed_base: 150.0,
            dot_radius: 5.0,
        };

        Self {
            addr: addr.to_string(),
            next_player_id: Arc::new(Mutex::new(1)),
            game_state: Arc::new(Mutex::new(GameState::new(constants))),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Phase 3: Broadcast current snapshot to all connected players
    pub async fn broadcast_state(&self) {
        let snapshot = {
            let gs = self.game_state.lock().await;
            gs.to_snapshot()
        };
        let msg = ServerMessage::StateUpdate(StateUpdateMessage { snapshot });
        let text = serde_json::to_string(&msg).unwrap();
        
        let conns = self.connections.lock().await;
        let tick = if let ServerMessage::StateUpdate(ref state) = msg {
            state.snapshot.tick
        } else {
            0
        };
        println!("[DEBUG] Broadcasting state to {} players (tick: {})", conns.len(), tick);
        for (id, tx) in conns.iter() {
            if tx.send(Message::Text(text.clone())).is_err() {
                println!("[DEBUG] Broadcast to player {} failed", id);
            } else {
                println!("[DEBUG] Successfully sent state update to player {}", id);
            }
        }
    }

    /// Phase 3: Accept new connections (renamed from run for clarity)
    pub async fn run_accept_loop(&self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        println!("Server WebSocket running at ws://{}/", self.addr);

        loop {
            let (stream, _) = listener.accept().await.unwrap();

            let id_counter = self.next_player_id.clone();
            let gs_state = self.game_state.clone();
            let connections = self.connections.clone();

            tokio::spawn(async move {
                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("Handshake failed: {:?}", e);
                        return;
                    }
                };

                let (mut ws_tx, mut ws_rx) = ws_stream.split();

                // Phase 3: Create a channel for sending messages to this client
                let (tx, mut rx): (Tx, Rx) = mpsc::unbounded_channel();

                // Phase 3: Forward messages from channel to websocket
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if ws_tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                // 1. 分配 player id
                let mut id_guard = id_counter.lock().await;
                let id = *id_guard;
                *id_guard += 1;
                drop(id_guard);

                println!("Player {} connected!", id);

                // 2. 加入 GameState
                gs_state.lock().await.add_player(id);

                // Phase 3: Register connection for broadcasting
                connections.lock().await.insert(id, tx);

                // 3. 读消息
                //    无论是 Close 还是错误，最后都会执行 remove_player
                while let Some(msg_result) = ws_rx.next().await {
                    match msg_result {
                        Ok(Message::Text(txt)) => {
                            println!("Raw text from {}: {}", id, txt);
                            match serde_json::from_str::<ClientMessage>(&txt) {
                                Ok(client_msg) => {
                                    println!("Parsed ClientMessage from {}: {:?}", id, client_msg);
                                    gs_state.lock().await.handle_message(id, client_msg);
                                }
                                Err(e) => {
                                    println!(
                                        "Failed to parse ClientMessage from {}: {:?}",
                                        id, e
                                    );
                                }
                            }
                        }
                        Ok(Message::Close(frame)) => {
                            println!("Player {} sent Close frame: {:?}", id, frame);
                            break;
                        }
                        Ok(other) => {
                            println!("Other WS message from {}: {:?}", id, other);
                        }
                        Err(e) => {
                            println!("WebSocket error from {}: {:?}", id, e);
                            break;
                        }
                    }
                }

                // 4. 无论如何，最终从 GameState 移除
                println!("Cleaning up player {} from GameState", id);
                gs_state.lock().await.remove_player(id);
                connections.lock().await.remove(&id);
            });
        }
    }
}
