// Collision detection and response
// Implements collide-and-slide algorithm for smooth player movement against voxel terrain

use crate::types::{BlockType, WorldPos, AABB};
use crate::physics::aabb::SweepResult;
use glam::Vec3;

/// Maximum number of collision iterations to prevent infinite loops
const MAX_COLLISION_ITERATIONS: usize = 4;

/// Minimum velocity threshold to stop sliding
const MIN_VELOCITY_THRESHOLD: f32 = 0.001;

/// Collision response information
#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    /// Final position after collision resolution
    pub position: Vec3,
    /// Final velocity after collision response
    pub velocity: Vec3,
    /// Whether the entity is on the ground
    pub on_ground: bool,
    /// Whether the entity hit a ceiling
    pub hit_ceiling: bool,
    /// Whether the entity hit a wall
    pub hit_wall: bool,
}

impl CollisionInfo {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self {
            position,
            velocity,
            on_ground: false,
            hit_ceiling: false,
            hit_wall: false,
        }
    }
}

/// Collide-and-slide algorithm
/// Moves an AABB through the world, sliding along surfaces when collisions occur
pub fn collide_and_slide<F>(
    aabb: &AABB,
    mut velocity: Vec3,
    get_block: F,
) -> CollisionInfo
where
    F: Fn(&WorldPos) -> BlockType,
{
    let mut position = aabb.center();
    let mut info = CollisionInfo::new(position, velocity);

    // Iterate to handle sliding along multiple surfaces
    for _ in 0..MAX_COLLISION_ITERATIONS {
        if velocity.length_squared() < MIN_VELOCITY_THRESHOLD * MIN_VELOCITY_THRESHOLD {
            break;
        }

        let current_aabb = AABB::from_center_size(position, aabb.size());

        match current_aabb.sweep_blocks(velocity, &get_block) {
            None => {
                // No collision - move freely
                position += velocity;
                break;
            }
            Some(sweep) => {
                // Collision detected

                // Move to point of collision
                position += velocity * sweep.time;

                // Detect ground/ceiling/wall based on normal
                // Normal.y > 0.5 means upward-facing surface (ground)
                // Normal.y < -0.5 means downward-facing surface (ceiling)
                if sweep.normal.y > 0.5 {
                    info.on_ground = true;
                } else if sweep.normal.y < -0.5 {
                    info.hit_ceiling = true;
                } else {
                    info.hit_wall = true;
                }

                // Small push away from surface to prevent getting stuck
                position += sweep.normal * 0.001;

                // Project remaining velocity along the collision surface
                let remaining_time = 1.0 - sweep.time;
                if remaining_time > 0.0 {
                    let remaining_velocity = velocity * remaining_time;

                    // Remove component of velocity in direction of normal
                    velocity = remaining_velocity - sweep.normal * remaining_velocity.dot(sweep.normal);
                } else {
                    break;
                }
            }
        }
    }

    info.position = position;
    info.velocity = velocity;
    info
}

/// Check if an AABB is standing on solid ground
pub fn is_on_ground<F>(aabb: &AABB, get_block: F) -> bool
where
    F: Fn(&WorldPos) -> BlockType,
{
    // Check slightly below the AABB
    let test_aabb = AABB {
        min: aabb.min - Vec3::new(0.0, 0.01, 0.0),
        max: aabb.max - Vec3::new(0.0, 0.01, 0.0),
    };

    let blocks = test_aabb.get_overlapping_blocks();

    for block_pos in blocks {
        let block = get_block(&block_pos);
        if block.is_solid() {
            let block_aabb = AABB::from_block(&block_pos);
            if test_aabb.intersects(&block_aabb) {
                // Check if the block is actually below us (block top near player bottom)
                if block_aabb.max.y >= aabb.min.y - 0.1 && block_aabb.max.y <= aabb.min.y + 0.05 {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if an AABB is hitting a ceiling
pub fn is_hitting_ceiling<F>(aabb: &AABB, get_block: F) -> bool
where
    F: Fn(&WorldPos) -> BlockType,
{
    // Check slightly above the AABB
    let test_aabb = AABB {
        min: aabb.min + Vec3::new(0.0, 0.01, 0.0),
        max: aabb.max + Vec3::new(0.0, 0.01, 0.0),
    };

    let blocks = test_aabb.get_overlapping_blocks();

    for block_pos in blocks {
        let block = get_block(&block_pos);
        if block.is_solid() {
            let block_aabb = AABB::from_block(&block_pos);
            if test_aabb.intersects(&block_aabb) {
                // Check if the block is actually above us (block bottom near player top)
                if block_aabb.min.y <= aabb.max.y + 0.1 && block_aabb.min.y >= aabb.max.y - 0.05 {
                    return true;
                }
            }
        }
    }

    false
}

/// Resolve AABB being stuck inside blocks (for when player spawns or glitches)
pub fn resolve_penetration<F>(aabb: &AABB, get_block: F) -> Vec3
where
    F: Fn(&WorldPos) -> BlockType,
{
    let blocks = aabb.get_overlapping_blocks();
    let mut total_correction = Vec3::ZERO;

    for block_pos in blocks {
        let block = get_block(&block_pos);
        if !block.is_solid() {
            continue;
        }

        let block_aabb = AABB::from_block(&block_pos);

        if let Some((normal, depth)) = aabb.get_penetration(&block_aabb) {
            // Push out along the normal by the penetration depth
            total_correction += normal * depth;
        }
    }

    total_correction
}

/// Smooth movement with collision for entities
pub fn move_with_collision<F>(
    aabb: &AABB,
    velocity: Vec3,
    delta_time: f32,
    get_block: F,
) -> CollisionInfo
where
    F: Fn(&WorldPos) -> BlockType,
{
    // Scale velocity by delta time to get frame movement
    let frame_velocity = velocity * delta_time;

    // Perform collide-and-slide
    let mut info = collide_and_slide(&aabb, frame_velocity, &get_block);

    // Check if we need to resolve any penetration (player might be stuck)
    let final_aabb = AABB::from_center_size(info.position, aabb.size());
    let correction = resolve_penetration(&final_aabb, get_block);

    if correction.length_squared() > 1e-6 {
        info.position += correction;
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world(pos: &WorldPos) -> BlockType {
        // Simple test world: ground at y=0, walls at x=±5, z=±5
        if pos.y < 0 {
            BlockType::Stone
        } else if pos.x.abs() > 5 || pos.z.abs() > 5 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_free_movement() {
        let aabb = AABB::from_center_size(Vec3::new(0.0, 5.0, 0.0), Vec3::splat(0.6));
        let velocity = Vec3::new(1.0, 0.0, 0.0);

        let info = collide_and_slide(&aabb, velocity, test_world);

        // Should move freely in air
        assert!((info.position.x - 1.0).abs() < 0.01);
        assert!(!info.on_ground);
    }

    #[test]
    fn test_ground_collision() {
        let aabb = AABB::from_center_size(Vec3::new(0.0, 0.3, 0.0), Vec3::splat(0.6));
        let velocity = Vec3::new(0.0, -1.0, 0.0);

        let info = collide_and_slide(&aabb, velocity, test_world);

        // Should hit ground and stop
        assert!(info.on_ground);
        assert!(info.position.y >= 0.3);
    }

    #[test]
    fn test_is_on_ground() {
        let aabb = AABB::from_center_size(Vec3::new(0.0, 0.3, 0.0), Vec3::splat(0.6));
        assert!(is_on_ground(&aabb, test_world));

        let floating = AABB::from_center_size(Vec3::new(0.0, 5.0, 0.0), Vec3::splat(0.6));
        assert!(!is_on_ground(&floating, test_world));
    }
}
