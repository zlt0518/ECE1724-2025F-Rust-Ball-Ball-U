# ECE1724-F1-2025F-Rust-Ball-Ball-U
ECE1724H F1 Special Topics in Software Engineering: Performant Software Systems with Rust 2025 Fall Project

**Course Link:** [ECE1724 F1: Special Topics in Software Systems: Performant Software Systems with Rust][course_page] \
**Project Link:** [BALL BALL U - Home Page][home_page]

## Team Members
| Team Member        | Student Number | Email                           |
|--------------------|----------------|---------------------------------|
| [Litao (John) Zhou][github_john] | 1006013092     | litao.zhou@mail.utoronto.ca |
| [Siyu Shao][github_siyu]         | 1007147204     | jasmine.shao@mail.utoronto.ca |
| [Chuyue Zhang][github_chuyue]    | 1005728303     | zhangchuyue.zhang@mail.utoronto.ca |

## Presentation and Demo
**Slide Link:** TBA!!!! \
**Presentation Link:** TBA!!!! \
**Demo Link:** TBA!!!! \

## Introduction 

**Ball Ball U** is a real-time multiplayer PvP game inspired by [Battle of Balls][battle_of_balls] and [Agar.io][agar_io], implemented in Rust with a focus on performance, concurrency, and fair competitive gameplay.


## Motivation

Agar.io and Battle of Balls are popular real-time multiplayer games that share the concept that players control balls that grow by consuming smaller balls and scattered food items. Their simple mechanics create deep competitive strategies, making it popular across a wide range of players. 

Our project **Ball Ball U** is inspired by both games, but shifts the focus from PvE-style survival to fast-paced PvP competition. In this version, players directly confront each other, testing both reflexes and strategic decisions such as movement and positioning. This change makes matches more dynamic and competitive, with outcomes driven by player interactions rather than environmental factors. 

We chose Rust because its performance, concurrency model, and memory safety make it well-suited for building a reliable real-time PvP multiplayer game. Although Rust has many existing resources for game development and for backend servers, there are very few complete examples that combine the two into a real-time multiplayer system. Our project helps fill this gap by showing how to connect a Rust game engine with an asynchronous server runtime, providing a clear reference for developers interested in building multiplayer games in Rust.

## Objective and key features

### Objective

The primary objective of this project is to design and implement a complete, end-to-end, real-time multiplayer game using a pure Rust technology stack.

Inspired by popular titles like Agar.io and Battle of Life, the primary goal is to create a clean, reusable, and well-documented architectural blueprint for client-server interaction within the Rust game development environment.

Our goal is to develop a high-performance, concurrent, and memory-safe multiplayer game server entirely in Rust. This real-time PvP project would include features such as rapid synchronization, consistent state, and low-latency scaling. To support this, we refine mechanics like collision detection, growth rules, and movement dynamics, ensuring fair and engaging encounters.

### Key Features

To achieve our objective, the project will be built around three core pillars: a centralized server, a responsive client, and a well-defined set of gameplay mechanics.

### Server

The server is the single source of truth for all game logic and state. It serves as the backend of the project. It has the following features:

 - Game state management: It manages the position, size, and velocity of all game objects, including all players and items in the game scene. 
 - Player input processing: It manages a dynamic list of WebSocket connections to each client. It receives and validates player actions sent from multiple clients via WebSockets. It would also resolve all the time sequential conflicts centrally.
 - Game mechanics engine: It continuously updates the game state in a fixed-tick loop, applying the core game mechanics at each step.
 - State Synchronization: It broadcasts a snapshot of the current game state to all clients at a regular interval.

The server would use Tokio Async Runtime as the core tech stack, and use the [Tokio-tungstenite][Tokio-tungstenite] to implement the WebSockets. 

### Client

The client is responsible for rendering the state received from the server and capturing the user input. It serves as the frontend of the project. It has the following features:

 - Graphics rendering: It utilizes the Bevy Engine to manage the game camera and render all the game objects on screen efficiently.
 - Server communication: It establishes a persistent [WebSocket][WebSocket] connection to the server to send player inputs and receive game state updates.
 - Input handling: It captures keyboard inputs and translates them into serialized messages for the server. To ensure responsive controls and hide network latency, the client would immediately act on keyboard inputs, providing immediate visual feedback. Meanwhile, the inputs are sent to the server for validation and processing.
 - User interface: It displays game information to the player, such as a real-time leaderboard and the names floating above each player's cell.

### Shared Game Mechanics Library

The shared game mechanics library is used to model the game objects and define the game mechanics in the game. The features include:

 - Game objects
   - Player cells: Each player controls a cell that contains a specific name, color, score, size, and speed.
   - Dots: These are small, static circles that spawn randomly on the map. Consuming them increases a player's score.
   - Game space: The game takes place within a large, rectangular area with defined boundaries.
 - Game mechanics:
   - When a player's cell collides with a dot, the dot is consumed, and its score is added to the player's score.
   - When a player's cell collides with another player's cell, the player with the higher score consumes the one with the lower score. The winner's score increases by the loser's score.
   - The size of the player is proportional to the score of the player.
   - The speed of the player is inversely proportional to the score of the player.
   - When a player is consumed, they are presented with an option to either rejoin the game or quit.
 - Serialization: For the data serialization during the message transmission, we would use [Serde][Serde] to do it. 
 - Communication protocol: We would define all the messages, including client messages and server messages, in the protocol here.

<div align="center">
  <p>
    <img src="documentations/images/ece1724_architecture.drawio.png" alt="architecture_diagram" width="70%">
  </p>
  <p>
    <em> Figure 1. Project Architecture Diagram </em>
  </p>
</div>

<div align="center">
  <p>
    <img src="documentations/images/ece1724_sequence_diagram.drawio.png" alt="sequence_diagram" width="70%">
  </p>
  <p>
    <em> Figure 2. Project Sequence Diagram </em>
  </p>
</div>

## Feature
The final deliverable of **Ball Ball U** is a real-time, multiplayer online game built entirely in **Rust**, utilising a **client–server architecture**.  
The main features are broken down as follows:
### 1. Robust Server Architecture
#### **Asynchronous Concurrency**
- Built on the **tokio** runtime and **tokio-tungstenite**.
- Handles multiple concurrent WebSocket connections asynchronously.
- Scales to many players without blocking threads.

#### **Authoritative Game Loop**
A deterministic **20 Hz tick loop (50ms)** enforces all game mechanics, including:
- Velocity-based movement physics.  
- Collision detection (Player ↔ Player, Player ↔ Dot).  
- Consuming/eating mechanics (dots, players), score updates, radius growth.  
- Player death & respawn logic.

#### **Message Queuing**
- Incoming packets (`ClientMessage`) are deserialized and queued asynchronously.
- Messages are processed *only during the next tick* to preserve order and fairness.

#### **Integrated HTTP Server**
- Runs concurrently with the WebSocket server using **hyper**.
- Serves static assets (HTML, JS, CSS) needed by the web client.

### 2. High-Performance Client

#### **Reactive Rendering Engine**
Uses **macroquad** to render at **60 FPS**, drawing:
- Player cells (unique colours, names).
- Food dots.
- UI overlays (Leaderboard, Timer, Start Menu, Connection status).

#### **Dynamic Camera System**
- Smooth camera tracking centred on the local player.
- Automatically adjusts viewport as the player moves.

#### **Input Management**
- Captures WASD/Arrow key movement, ESC (quit), Enter (ready).
- Serialises actions into `ClientMessage` packets sent to the server.

#### **State Synchronisation**
Client loop:
- Capture Input → Send Command → Receive Snapshot → Render
- Client updates its state based on authoritative  `ServerMessage::StateUpdate` snapshots.

### 3. Networking & Data Consistency

#### **Shared Protocol Library**
A shared crate ensures both client & server use:
- Identical structs (`PlayerSpec`, `Dot`, `GameSnapshot`, etc.)
- Shared physics constants  
→ Prevents desynchronisation.

#### **JSON Serialisation**
Using **serde** and **serde_json**.

**Client sends:**
- `Join`
- `Move` (direction + sequence number)
- `Ready`
- `Quit`

**Server broadcasts:**
- `Welcome` (initial setup for joining players)
- `StateUpdate` (full world snapshot each tick)
- `Bye` (disconnect notice)

#### **Global Broadcasting**
- Once per tick, the server serialises the full game state.
- Broadcasts via Tx channels to all connected clients.
- Ensures every player receives a synchronised world state.

## User’s (or Developer’s) Guide

This project consists of three Rust crates: **server**, **client**, and **shared**.  
- The shared crate defines all protocol messages, game objects, and mechanics.  
- The server crate provides the authoritative game simulation and networking
- The client crate provides rendering and input handling.

To use the crates in your own Rust project, include them in your workspace’s `Cargo.toml`:

```toml
[workspace]
members = ["server", "client", "shared"]
```

Example usage of the **shared** crate types:

```rust
extern crate shared;

use shared::protocol::{ClientMessage, ServerMessage};

fn main() {
    let msg = ClientMessage::Hello;
    println!("Client message created: {:?}", msg);
}
```

Example usage of the **server** crate’s game loop module:

```rust
extern crate server;

fn main() {
    println!("Starting server game loop...");
    server::game_loop::start_game_loop();
}
```

Example usage of the **client** crate:

```rust
extern crate client;

fn main() {
    println!("Launching client...");
    client::start_client();
}
```




## Reproducibility Guide

To reproduce the build and run the project exactly as intended, follow these steps with no deviation.
These instructions work on both Ubuntu Linux and macOS Sonoma.

### 0. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 1. Build the entire workspace
```bash
cargo build
```
This compiles all three crates: server, client, and shared.

### 1.5 Network ports configuration

**The server process opens two TCP ports:**
- An HTTP server for static assets (HTML/JS/CSS) on port <34567>.
- A WebSocket game server for real-time gameplay on port <34568>.

**To run the project successfully:**
- Make sure these two ports (<34567>, <34568>) are not already in use on the machine where you run the server.
- If there is a firewall, allow incoming TCP connections on these ports (at least from the machine running the client).

### 2. Run the server
```bash
cargo run -p server
```
The server will start listening for WebSocket connections and begin running the game loop.

### 2.5 (Optional): Connect From Another Machine Using SSH

If you want to run the server on one machine and the client on another:

On the server machine, find its IP address:
```bash
ip addr
```
or on macOS:
```bash
ifconfig
```

On the second machine, connect via SSH:
```bash
ssh username@SERVER_IP_ADDRESS
```

After connecting, you can run the server or client normally on that machine.

### 3. Run the client

Open a second terminal window (or use a second machine via SSH) and run:
```bash
cargo run -p client
```

The client will automatically connect to the server’s WebSocket endpoint and begin rendering the game.
**You may open multiple client instances (each in its own terminal or via SSH), and as long as they are connected to the same server address, all clients will join the same game world and play together**

### 4. Controls and UI

#### When the client window opens:
- Use WASD or the Arrow Keys to move your ball.
- Keyboard input player nickname.
- Press Enter to mark yourself as ready / start the game.
- Press Esc to quit the client.

#### The client shows:
- Your own ball (with a unique colour and name),
- Other players’ balls,
- Food dots,
- Basic UI overlays, such as a timer and scores.


## Contributions by each team member
### Siyu Shao: 
### Litao(John) Zhou:
### Chuyue Zhang: 


## Lesson Learned
Working on Ball Ball U let us use Rust in a way that is very different from small homework problems. We had to keep an authoritative game server, a shared-protocol crate, and a real-time client working together. Through this process, we became more comfortable with Rust’s ownership rules, async/await, and WebSocket networking, and we saw how these tools can be combined to support a fast PvP game.

At the same time, the project also showed us several things we would do differently next time. Early on, we spent a lot of time exploring text-based rendering, and only later switched fully to a graphical engine. If we had made this decision earlier, we could have put more effort into gameplay polish and user experience. On the server side, some modules became more complex than we expected, and we learned that agreeing on clear boundaries between networking, game logic, and state management at the design stage would make later changes much easier.

Overall, this project has been a very positive learning experience for our team, both technically and in how we plan and divide work in a longer project. We now have a working real-time multiplayer game written entirely in Rust, with an authoritative server, a graphical client, and a shared protocol that keeps them in sync. We feel that Ball Ball U adds a small but useful example to the Rust ecosystem for developers who want to build real-time multiplayer games and are looking for an end-to-end reference.

[course_page]: https://www.eecg.toronto.edu/~bli/ece1724
[home_page]: https://github.com/zlt0518/ECE1724-2025F-Rust-Ball-Ball-U
[battle_of_balls]: https://www.battleofballs.com
[agar_io]: https://agar.io
[github_john]: https://github.com/zlt0518
[github_siyu]: https://github.com/jassiyu
[github_chuyue]: https://github.com/IronDumpling

[Tokio-tungstenite]:https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/
[WebSocket]: https://github.com/snapview/tungstenite-rs
[Serde]: https://serde.rs/

