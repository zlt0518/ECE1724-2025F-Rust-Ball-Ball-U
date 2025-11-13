mod websocket_manager;
mod game_state;
mod game_loop;

use websocket_manager::WebSocketManager;

#[tokio::main]
async fn main() {
    let ws = WebSocketManager::new("127.0.0.1:8000").await;
    ws.run().await;
}
