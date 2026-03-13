// Raycasting through voxel grid using DDA (Digital Differential Analyzer) algorithm
// Used for block targeting, placing, and breaking

use crate::types::{BlockType, Direction, WorldPos};
use glam::Vec3;

/// Maximum distance for raycasting in blocks
pub const MAX_RAYCAST_DISTANCE: f32 = 5.0;

/// Result of a raycast
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// Position of the block that was hit
    pub block_pos: WorldPos,
    /// Position of the block face that was hit (for placing blocks)
    pub adjacent_pos: WorldPos,
    /// Which face of the block was hit
    pub face: Direction,
    /// Distance from ray origin to hit point
    pub distance: f32,
    /// Exact hit position in world space
    pub hit_point: Vec3,
}

impl RaycastHit {
    pub fn new(
        block_pos: WorldPos,
        adjacent_pos: WorldPos,
        face: Direction,
        distance: f32,
        hit_point: Vec3,
    ) -> Self {
        Self {
            block_pos,
            adjacent_pos,
            face,
            distance,
            hit_point,
        }
    }
}

/// DDA raycasting through voxel grid
/// Returns the first solid block hit, along with hit information
pub fn raycast<F>(origin: Vec3, direction: Vec3, max_distance: f32, get_block: F) -> Option<RaycastHit>
where
    F: Fn(&WorldPos) -> BlockType,
{
    if direction.length_squared() < 1e-8 {
        return None;
    }

    let direction = direction.normalize();

    // Current position in world coordinates
    let mut pos = origin;

    // Current voxel coordinates
    let mut voxel = WorldPos::from_vec3(pos);

    // Step direction for each axis (-1, 0, or 1)
    let step_x = direction.x.signum() as i32;
    let step_y = direction.y.signum() as i32;
    let step_z = direction.z.signum() as i32;

    // Distance to travel along ray to cross one voxel boundary on each axis
    let delta_x = if direction.x.abs() > 1e-8 {
        (1.0 / direction.x).abs()
    } else {
        f32::INFINITY
    };
    let delta_y = if direction.y.abs() > 1e-8 {
        (1.0 / direction.y).abs()
    } else {
        f32::INFINITY
    };
    let delta_z = if direction.z.abs() > 1e-8 {
        (1.0 / direction.z).abs()
    } else {
        f32::INFINITY
    };

    // Distance along ray to next voxel boundary
    let mut t_max_x = if direction.x.abs() > 1e-8 {
        let next_boundary = if step_x > 0 {
            voxel.x as f32 + 1.0
        } else {
            voxel.x as f32
        };
        (next_boundary - pos.x) / direction.x
    } else {
        f32::INFINITY
    };

    let mut t_max_y = if direction.y.abs() > 1e-8 {
        let next_boundary = if step_y > 0 {
            voxel.y as f32 + 1.0
        } else {
            voxel.y as f32
        };
        (next_boundary - pos.y) / direction.y
    } else {
        f32::INFINITY
    };

    let mut t_max_z = if direction.z.abs() > 1e-8 {
        let next_boundary = if step_z > 0 {
            voxel.z as f32 + 1.0
        } else {
            voxel.z as f32
        };
        (next_boundary - pos.z) / direction.z
    } else {
        f32::INFINITY
    };

    // Track which face we entered the current voxel from
    let mut face = Direction::Up;
    let mut distance = 0.0;

    // DDA traversal
    loop {
        // Check if we've exceeded max distance
        if distance > max_distance {
            return None;
        }

        // Check current voxel
        let block = get_block(&voxel);
        if block.is_solid() {
            // Hit a solid block!
            let hit_point = origin + direction * distance;

            // Calculate adjacent position (for block placement)
            let offset = face.opposite().offset();
            let adjacent_pos = WorldPos::new(
                voxel.x + offset.x,
                voxel.y + offset.y,
                voxel.z + offset.z,
            );

            return Some(RaycastHit::new(
                voxel,
                adjacent_pos,
                face,
                distance,
                hit_point,
            ));
        }

        // Step to next voxel
        if t_max_x < t_max_y && t_max_x < t_max_z {
            voxel.x += step_x;
            distance = t_max_x;
            t_max_x += delta_x;
            face = if step_x > 0 {
                Direction::West
            } else {
                Direction::East
            };
        } else if t_max_y < t_max_z {
            voxel.y += step_y;
            distance = t_max_y;
            t_max_y += delta_y;
            face = if step_y > 0 {
                Direction::Down
            } else {
                Direction::Up
            };
        } else {
            voxel.z += step_z;
            distance = t_max_z;
            t_max_z += delta_z;
            face = if step_z > 0 {
                Direction::North
            } else {
                Direction::South
            };
        }

        // Safety check to prevent infinite loops
        if distance > max_distance * 2.0 {
            return None;
        }
    }
}

/// Simpler raycast that only returns the block position and face
pub fn raycast_block<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    get_block: F,
) -> Option<(WorldPos, Direction)>
where
    F: Fn(&WorldPos) -> BlockType,
{
    raycast(origin, direction, max_distance, get_block)
        .map(|hit| (hit.block_pos, hit.face))
}

/// Check if there's a clear line of sight between two points
pub fn has_line_of_sight<F>(from: Vec3, to: Vec3, get_block: F) -> bool
where
    F: Fn(&WorldPos) -> BlockType,
{
    let direction = to - from;
    let distance = direction.length();

    if distance < 1e-8 {
        return true;
    }

    // Raycast from 'from' to 'to'
    match raycast(from, direction, distance, get_block) {
        None => true, // No obstruction
        Some(hit) => hit.distance >= distance, // Hit is beyond target
    }
}

/// Get all blocks along a ray up to max_distance
pub fn get_blocks_along_ray<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    get_block: F,
) -> Vec<(WorldPos, BlockType)>
where
    F: Fn(&WorldPos) -> BlockType,
{
    let mut blocks = Vec::new();

    if direction.length_squared() < 1e-8 {
        return blocks;
    }

    let direction = direction.normalize();
    let mut pos = origin;
    let mut voxel = WorldPos::from_vec3(pos);
    let mut visited = std::collections::HashSet::new();

    let step_x = direction.x.signum() as i32;
    let step_y = direction.y.signum() as i32;
    let step_z = direction.z.signum() as i32;

    let delta_x = if direction.x.abs() > 1e-8 {
        (1.0 / direction.x).abs()
    } else {
        f32::INFINITY
    };
    let delta_y = if direction.y.abs() > 1e-8 {
        (1.0 / direction.y).abs()
    } else {
        f32::INFINITY
    };
    let delta_z = if direction.z.abs() > 1e-8 {
        (1.0 / direction.z).abs()
    } else {
        f32::INFINITY
    };

    let mut t_max_x = if direction.x.abs() > 1e-8 {
        let next_boundary = if step_x > 0 {
            voxel.x as f32 + 1.0
        } else {
            voxel.x as f32
        };
        (next_boundary - pos.x) / direction.x
    } else {
        f32::INFINITY
    };

    let mut t_max_y = if direction.y.abs() > 1e-8 {
        let next_boundary = if step_y > 0 {
            voxel.y as f32 + 1.0
        } else {
            voxel.y as f32
        };
        (next_boundary - pos.y) / direction.y
    } else {
        f32::INFINITY
    };

    let mut t_max_z = if direction.z.abs() > 1e-8 {
        let next_boundary = if step_z > 0 {
            voxel.z as f32 + 1.0
        } else {
            voxel.z as f32
        };
        (next_boundary - pos.z) / direction.z
    } else {
        f32::INFINITY
    };

    let mut distance = 0.0;

    while distance <= max_distance {
        if !visited.contains(&voxel) {
            let block = get_block(&voxel);
            blocks.push((voxel, block));
            visited.insert(voxel);
        }

        if t_max_x < t_max_y && t_max_x < t_max_z {
            voxel.x += step_x;
            distance = t_max_x;
            t_max_x += delta_x;
        } else if t_max_y < t_max_z {
            voxel.y += step_y;
            distance = t_max_y;
            t_max_y += delta_y;
        } else {
            voxel.z += step_z;
            distance = t_max_z;
            t_max_z += delta_z;
        }

        if distance > max_distance * 2.0 {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world(pos: &WorldPos) -> BlockType {
        // Wall at x=5
        if pos.x == 5 {
            BlockType::Stone
        } else {
            BlockType::Air
        }
    }

    #[test]
    fn test_raycast_hit() {
        let origin = Vec3::ZERO;
        let direction = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycast(origin, direction, 10.0, test_world);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert_eq!(hit.block_pos.x, 5);
        assert_eq!(hit.face, Direction::West);
    }

    #[test]
    fn test_raycast_miss() {
        let origin = Vec3::ZERO;
        let direction = Vec3::new(-1.0, 0.0, 0.0); // Looking away from wall

        let hit = raycast(origin, direction, 10.0, test_world);
        assert!(hit.is_none());
    }

    #[test]
    fn test_line_of_sight() {
        let from = Vec3::ZERO;
        let to = Vec3::new(4.0, 0.0, 0.0); // Before wall
        assert!(has_line_of_sight(from, to, test_world));

        let blocked = Vec3::new(6.0, 0.0, 0.0); // Beyond wall
        assert!(!has_line_of_sight(from, blocked, test_world));
    }
}
