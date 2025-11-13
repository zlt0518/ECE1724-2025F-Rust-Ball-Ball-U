use crate::objects::{PlayerSpec, Dot};

/// Distance between two points
fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}


// =============================================
// 1. cellsCollisionsCheck: player-player collision?
// =============================================

/// Returns true if two players' circles overlap
pub fn cells_collisions_check(a: &PlayerSpec, b: &PlayerSpec) -> bool {
    let d = distance(a.x, a.y, b.x, b.y);
    d < (a.radius + b.radius)
}


// =============================================
// 2. dotCollisionCheck: player eats dot?
// =============================================

/// Returns true if player overlaps with the dot
pub fn dot_collision_check(player: &PlayerSpec, dot: &Dot) -> bool {
    let d = distance(player.x, player.y, dot.x, dot.y);
    d < (player.radius + dot.radius)
}


// =============================================
// 3. calculateSpeedFromScore
// =============================================

/// Speed decreases slowly as player grows larger.
/// At score 0 => base_speed
/// Larger mass => slightly slower movement
pub fn calculate_speed_from_score(score: u32, base_speed: f32) -> f32 {
    let slow_factor = 1.0 / (1.0 + (score as f32) * 0.005);
    base_speed * slow_factor
}


// =============================================
// 4. calculateRadiusFromScore
// =============================================

/// Radius grows as sqrt(score), typical agar.io mechanic.
pub fn calculate_radius_from_score(score: u32, base_radius: f32) -> f32 {
    base_radius + (score as f32).sqrt()
}


// =============================================
// 5. updatePosition: update player movement
// =============================================

/// Moves a player toward target input direction.
/// dx, dy should be normalized values from client.
///
/// Example:
///    - W pressed => (0, -1)
///    - A pressed => (-1, 0)
///    - W + A => normalized
pub fn update_position(
    player: &mut PlayerSpec,
    dx: f32,
    dy: f32,
    speed: f32,
    delta_time_ms: f32
) {
    // Normalize input direction
    let mag = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if mag > 0.0 {
        (dx / mag, dy / mag)
    } else {
        (0.0, 0.0)
    };

    // Distance = speed * time
    let dt_sec = delta_time_ms / 1000.0;

    player.x += nx * speed * dt_sec;
    player.y += ny * speed * dt_sec;
}


// =============================================
// 6. consumeCalculation: size updates after consuming
// =============================================

/// When big_player eats small_player:
/// - new score = sum
/// - new radius = recalculated from score
pub fn consume_calculation(
    big_player: &mut PlayerSpec,
    small_player: &PlayerSpec,
    base_radius: f32
) {
    big_player.score += small_player.score;
    big_player.radius = calculate_radius_from_score(big_player.score, base_radius);
}
