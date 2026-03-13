// Simple pathfinding for mobs
// Provides basic navigation toward a target with obstacle avoidance

use glam::Vec3;
use crate::types::{BlockType, WorldPos};

/// Simple pathfinding - returns a position to move toward
/// This is a basic implementation that:
/// - Moves directly toward target if path is clear
/// - Tries to go around obstacles
/// - Can jump up 1-block ledges
/// - Avoids falling off cliffs
pub fn simple_pathfind<F>(current_pos: Vec3, target_pos: Vec3, get_block: F) -> Vec3
where
    F: Fn(&WorldPos) -> BlockType,
{
    // Direct path to target
    let direct = target_pos - current_pos;
    let distance = direct.length();

    if distance < 0.5 {
        return target_pos; // Already at target
    }

    let direction = direct.normalize();

    // Check if direct path is clear
    let check_ahead = current_pos + direction * 0.5;
    let block_ahead = get_block(&WorldPos::from_vec3(check_ahead));

    if !block_ahead.is_solid() {
        // Check for ledges (don't walk off cliffs)
        let check_below = check_ahead - Vec3::new(0.0, 2.0, 0.0);
        let block_below = get_block(&WorldPos::from_vec3(check_below));

        if block_below.is_solid() {
            // Safe to move forward
            return check_ahead;
        } else {
            // Cliff detected, try to go around
            return try_go_around(current_pos, target_pos, direction, &get_block);
        }
    } else {
        // Obstacle ahead, check if we can jump over it
        let block_above_obstacle = get_block(&WorldPos::from_vec3(check_ahead + Vec3::Y));

        if !block_above_obstacle.is_solid() {
            // Can jump over 1-block obstacle
            return check_ahead + Vec3::Y;
        } else {
            // Too tall to jump, go around
            return try_go_around(current_pos, target_pos, direction, &get_block);
        }
    }
}

/// Try to navigate around an obstacle
fn try_go_around<F>(
    current_pos: Vec3,
    target_pos: Vec3,
    forward: Vec3,
    get_block: F,
) -> Vec3
where
    F: Fn(&WorldPos) -> BlockType,
{
    // Try perpendicular directions
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let left = -right;

    // Test right side
    let right_pos = current_pos + right * 0.5;
    let right_block = get_block(&WorldPos::from_vec3(right_pos));

    if !right_block.is_solid() {
        let below_right = right_pos - Vec3::new(0.0, 1.0, 0.0);
        if get_block(&WorldPos::from_vec3(below_right)).is_solid() {
            // Right is safe
            return right_pos;
        }
    }

    // Test left side
    let left_pos = current_pos + left * 0.5;
    let left_block = get_block(&WorldPos::from_vec3(left_pos));

    if !left_block.is_solid() {
        let below_left = left_pos - Vec3::new(0.0, 1.0, 0.0);
        if get_block(&WorldPos::from_vec3(below_left)).is_solid() {
            // Left is safe
            return left_pos;
        }
    }

    // Calculate which side is closer to target
    let right_distance = (right_pos - target_pos).length_squared();
    let left_distance = (left_pos - target_pos).length_squared();

    if right_distance < left_distance {
        right_pos
    } else {
        left_pos
    }
}

/// Check if there's a clear line of sight between two positions
pub fn has_line_of_sight<F>(from: Vec3, to: Vec3, get_block: F) -> bool
where
    F: Fn(&WorldPos) -> BlockType,
{
    let direction = to - from;
    let distance = direction.length();
    let step = direction.normalize() * 0.5;

    let mut current = from;
    let mut traveled = 0.0;

    while traveled < distance {
        let block = get_block(&WorldPos::from_vec3(current));
        if block.is_solid() {
            return false;
        }

        current += step;
        traveled += 0.5;
    }

    true
}

/// Find a valid spawn position near a target position
/// Ensures the position has solid ground and air above
pub fn find_valid_spawn_position<F>(
    center: Vec3,
    radius: f32,
    get_block: F,
) -> Option<Vec3>
where
    F: Fn(&WorldPos) -> BlockType,
{
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Try up to 20 random positions
    for _ in 0..20 {
        let offset = Vec3::new(
            rng.gen_range(-radius..radius),
            rng.gen_range(-3.0..3.0),
            rng.gen_range(-radius..radius),
        );

        let test_pos = center + offset;

        // Check if position has solid ground below
        let ground_pos = WorldPos::from_vec3(test_pos - Vec3::Y);
        let ground_block = get_block(&ground_pos);

        if !ground_block.is_solid() {
            continue;
        }

        // Check if there's air at spawn position
        let spawn_block = get_block(&WorldPos::from_vec3(test_pos));
        if spawn_block.is_solid() {
            continue;
        }

        // Check if there's air above (2 blocks high)
        let head_block = get_block(&WorldPos::from_vec3(test_pos + Vec3::Y));
        if head_block.is_solid() {
            continue;
        }

        // Valid spawn position found
        return Some(test_pos);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_world(pos: &WorldPos) -> BlockType {
        // Simple test world: ground at y=0, air above
        if pos.y <= 0 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_direct_pathfind() {
        let from = Vec3::new(0.0, 1.0, 0.0);
        let to = Vec3::new(5.0, 1.0, 0.0);

        let result = simple_pathfind(from, to, mock_world);

        // Should move toward target
        assert!(result.x > from.x);
    }

    #[test]
    fn test_line_of_sight_clear() {
        let from = Vec3::new(0.0, 1.0, 0.0);
        let to = Vec3::new(5.0, 1.0, 0.0);

        let result = has_line_of_sight(from, to, mock_world);
        assert!(result);
    }

    #[test]
    fn test_line_of_sight_blocked() {
        let from = Vec3::new(0.0, 1.0, 0.0);
        let to = Vec3::new(5.0, 1.0, 0.0);

        let blocked_world = |pos: &WorldPos| {
            if pos.x == 2 && pos.y == 1 {
                BlockType::Stone
            } else {
                mock_world(pos)
            }
        };

        let result = has_line_of_sight(from, to, blocked_world);
        assert!(!result);
    }

    #[test]
    fn test_find_valid_spawn() {
        let center = Vec3::new(0.0, 5.0, 0.0);

        let result = find_valid_spawn_position(center, 10.0, mock_world);

        assert!(result.is_some());
        if let Some(pos) = result {
            // Should be above ground
            assert!(pos.y > 0.0);
        }
    }
}
