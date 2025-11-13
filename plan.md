# Ball Ball U - Implementation Plan

## Project Structure

```
ball-ball-u/
├── Cargo.toml (workspace)
├── server/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── game_loop.rs
│       ├── websocket_manager.rs
│       └── game_state.rs
├── client/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── input_manager.rs
│       ├── render_manager.rs
│       └── websocket.rs
└── shared/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── objects.rs
        ├── mechanics.rs
        └── protocol.rs
```

## Team Task Attribution

### Teammate 1: Server Backend (Primary Owner)

**Phase 1: Server Foundation**

- Set up Cargo workspace with `server/` crate
- Add dependencies: `tokio`, `tokio-tungstenite`, `serde`, `serde_json`
- Implement basic WebSocket server that accepts connections
- Create player connection registry (HashMap<u64, WebSocket>)

**Phase 2: Game State Management**

- Implement `GameState` struct holding all players and dots
- Add player join/quit handling
- Implement state serialization to `GameSnapshot` messages

**Phase 3: Game Loop**

- Create fixed-tick game loop (e.g., 20 ticks/second using `tokio::time::interval`)
- Process player input queues each tick
- Apply game mechanics (collision detection, consumption)
- Broadcast state updates to all connected clients

**Phase 4: Integration & Testing**

- Test with multiple concurrent connections
- Handle edge cases (disconnections, network delays)
- Add basic logging

### Teammate 2: Client Frontend (Primary Owner)

**Phase 1: Client Foundation**

- Set up `client/` crate with Bevy dependencies
- Add `tokio-tungstenite` for WebSocket client
- Implement basic Bevy app with empty game window

**Phase 2: WebSocket Connection**

- Create WebSocket client connecting to server
- Implement message sending (UserInput)
- Implement message receiving (GameSnapshot, StateUpdate)
- Store received game state in Bevy resources

**Phase 3: Rendering**

- Implement `SceneRenderer` system:
  - Render player cells as colored circles with names
  - Render dots as small circles
  - Implement camera following local player
- Add background and game space boundaries

**Phase 4: Input & UI**

- Capture WASD/Arrow key inputs
- Send inputs to server with sequence numbers
- Implement client-side prediction for smooth movement
- Add UI overlay: leaderboard, player score, rejoin/quit menu

### Teammate 3: Shared Library & Integration (Primary Owner)

**Phase 1: Core Data Structures**

- Set up `shared/` crate as library
- Define game objects in `objects.rs`:
  - `PlayerCell` (id, name, color, score, size, speed, position)
  - `Dot` (id, score, color, size, position)
  - `GameSpace` (width, length)
- Define `GameConstant` (TICK_INTERVAL, COLLIDE_SIZE_FRAC, etc.)

**Phase 2: Game Mechanics**

- Implement collision detection functions in `mechanics.rs`:
  - `CellCollisionCheck` - check if two cells collide
  - `DotCollisionCheck` - check if cell collides with dot
  - `CellSpeedCalculation` - calculate speed from score
  - `CellSizeCalculation` - calculate size from score
  - `CellPositionCalculation` - update position based on input
  - `CellScoreCalculation` - handle score changes on consumption

**Phase 3: Communication Protocol**

- Define protocol in `protocol.rs`:
  - `ClientMessage` enum: Join, Input, Quit
  - `ServerMessage` enum: Welcome, StateUpdate, Bye
  - `UserInput` struct (dx, dy, sequence_number)
  - `GameSnapshot` struct (players, dots, sequence_number)
  - Derive `Serialize` and `Deserialize` for all

**Phase 4: Integration Support**

- Help Teammate 1 integrate mechanics into server game loop
- Help Teammate 2 integrate rendering with game objects
- Write basic integration tests
- Document the shared API with examples

## Development Workflow

1. **Week 1**: All teammates set up their respective crates, basic structure
2. **Week 2**: 

   - Teammate 1: WebSocket server + player registry
   - Teammate 2: Bevy app + WebSocket client
   - Teammate 3: Complete data structures + protocol

3. **Week 3**: 

   - Teammate 1: Game loop with mechanics integration
   - Teammate 2: Rendering implementation
   - Teammate 3: Game mechanics implementation

4. **Week 4**: 

   - Teammate 1: State broadcasting
   - Teammate 2: Input handling + prediction
   - Teammate 3: Integration testing

5. **Week 5**: Integration, bug fixes, polish

## Key Technical Decisions

1. **State Authority**: Server is authoritative; client predictions are cosmetic only
2. **Serialization**: Use JSON via `serde_json` for simplicity (binary formats like bincode for optimization if needed)
3. **Tick Rate**: 20 ticks/second for server, 60 fps for client rendering
4. **ID Generation**: Server assigns unique u64 IDs to players and dots
5. **Collision**: Simple circle-circle collision using distance calculation
6. **Respawn**: On death, server removes player; client shows rejoin menu

## Dependencies

**Server** (`server/Cargo.toml`):

```toml
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde_json = "1.0"
```

**Client** (`client/Cargo.toml`):

```toml
bevy = "0.12"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde_json = "1.0"
```

**Shared** (`shared/Cargo.toml`):

```toml
serde = { version = "1.0", features = ["derive"] }
```

## Minimal Feature Set (MVP)

For a course project, focus on these essentials:

- ✓ 2-4 players can connect and see each other
- ✓ Players can move using keyboard
- ✓ Basic collision and consumption mechanics
- ✓ Simple circular rendering
- ✓ Score display
- ✓ Respawn on death

**Defer these for later** (if time permits):

- Advanced UI polish
- Sound effects
- Complex animations
- Cell splitting mechanics
- Spectator mode
- Persistence/database