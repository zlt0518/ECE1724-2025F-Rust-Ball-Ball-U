use serde::{Serialize, Deserialize};
use crate::{GameConstant, GameSnapshot};


/// Client → Server Messages
/// Client input command (WASD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    pub dx: f32,              // movement direction x
    pub dy: f32,              // movement direction y
    pub sequence_number: u64, // ensures ordering
}

/// Messages sent from the client to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// New client wants to join with a username
    Join { name: String },

    /// Movement input (continuous, deprecated in favor of Move)
    Input { input: UserInput },

    /// Discrete movement: move a fixed distance in direction
    Move { dx: f32, dy: f32, distance: f32 },

    /// Player is ready to start the game (pressed space)
    Ready,

    /// Client gracefully disconnects
    Quit,
}


/// Server → Client Messages
/// Sent to client immediately after connection accepted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeMessage {
    pub player_id: u64,           // assigned by server
    pub constants: GameConstant,  // game constants
}

/// Normal broadcast update from server every tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdateMessage {
    pub snapshot: GameSnapshot,   // full world snapshot
}

/// Server instructs client to exit or rejoin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByeMessage {
    pub reason: String,
}


/// Enum of all possible server → client packets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome(WelcomeMessage),
    StateUpdate(StateUpdateMessage),
    Bye(ByeMessage),
}
