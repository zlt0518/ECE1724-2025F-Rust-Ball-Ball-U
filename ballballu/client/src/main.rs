use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8000";
    println!("Connecting to {}", url);

    let (ws_stream, _) = match connect_async(url).await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to connect: {:?}", e);
            return;
        }
    };

    println!("Connected to server!");

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to print messages from the server
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("[Server] {}", text);
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed connection.");
                    break;
                }
                _ => {}
            }
        }
    });

    // Read user input and send to server
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    println!("Type messages to send to server:");

    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        if line == "/quit" {
            println!("Closing connection…");
            let _ = write.send(Message::Close(None)).await;
            break;
        }

        write.send(Message::Text(line)).await.unwrap();
    }
}
