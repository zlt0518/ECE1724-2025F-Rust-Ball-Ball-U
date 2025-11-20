use std::time::Instant;
use shared::GameSnapshot;

#[derive(Clone)]
pub struct ClientSnapshot {
    pub snapshot: GameSnapshot,
    pub received_at: Instant,
}