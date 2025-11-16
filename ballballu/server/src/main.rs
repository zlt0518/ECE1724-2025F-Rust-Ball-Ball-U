mod websocket_manager;
mod game_state;
mod game_loop;

use std::sync::Arc;
use websocket_manager::WebSocketManager;
use game_loop::GameLoop;

#[tokio::main]
async fn main() {
    let ws = Arc::new(WebSocketManager::new("127.0.0.1:8000").await);
    let game_loop = GameLoop::new(ws.clone());

    // Phase 3: Spawn accept loop in background
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        ws_clone.run_accept_loop().await;
    });

    // Phase 3: Run game loop (blocks here)
    game_loop.run().await;
}
