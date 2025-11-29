mod websocket_manager;
mod game_state;
mod game_loop;
mod http_server;

use std::sync::Arc;
use std::path::PathBuf;
use websocket_manager::WebSocketManager;
use game_loop::GameLoop;
use http_server::HttpServer;

#[tokio::main]
async fn main() {
    // WebSocket server for game communication
    let ws = Arc::new(WebSocketManager::new("128.100.8.107:34568").await);
    let game_loop = GameLoop::new(ws.clone());

    // HTTP server for static files (test.html, styles.css, app.js)
    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let http_server = HttpServer::new("128.100.8.107:34567", static_dir);

    // Spawn WebSocket accept loop
    let ws_clone = ws.clone();
    tokio::spawn(async move {
        ws_clone.run_accept_loop().await;
    });

    // Spawn HTTP server for static files
    let http_server_clone = http_server;
    tokio::spawn(async move {
        http_server_clone.run().await;
    });

    // Run game loop (blocks here)
    game_loop.run().await;
}
