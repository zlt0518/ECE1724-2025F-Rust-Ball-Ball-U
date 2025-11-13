use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use shared::protocol::ClientMessage;
use crate::game_state::GameState;

pub struct WebSocketManager {
    pub addr: String,
    pub next_player_id: Arc<Mutex<u64>>,
    pub game_state: Arc<Mutex<GameState>>,
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
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        println!("Server WebSocket running at ws://{}/", self.addr);

        loop {
            let (stream, _) = listener.accept().await.unwrap();

            let id_counter = self.next_player_id.clone();
            let gs_state = self.game_state.clone();

            tokio::spawn(async move {
                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        println!("Handshake failed: {:?}", e);
                        return;
                    }
                };

                let mut ws_stream = ws_stream;

                // 1. 分配 player id
                let mut id_guard = id_counter.lock().await;
                let id = *id_guard;
                *id_guard += 1;
                drop(id_guard);

                println!("Player {} connected!", id);

                // 2. 加入 GameState
                gs_state.lock().await.add_player(id);

                // 3. 读消息
                //    无论是 Close 还是错误，最后都会执行 remove_player
                while let Some(msg_result) = ws_stream.next().await {
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
            });
        }
    }
}
