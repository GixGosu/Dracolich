// AABB (Axis-Aligned Bounding Box) collision detection and sweep tests

use crate::types::{BlockType, WorldPos, AABB};
use glam::Vec3;

/// Result of a sweep test
#[derive(Debug, Clone, Copy)]
pub struct SweepResult {
    /// Time of impact (0.0 to 1.0, where 1.0 is the full movement)
    pub time: f32,
    /// Normal of the surface hit
    pub normal: Vec3,
    /// Position where collision occurred
    pub position: Vec3,
}

impl SweepResult {
    pub fn new(time: f32, normal: Vec3, position: Vec3) -> Self {
        Self {
            time,
            normal,
            position,
        }
    }
}

/// Extended AABB functionality for collision detection
impl AABB {
    /// Sweep this AABB against a static AABB
    /// Returns the time of impact (0.0 to 1.0) and collision normal
    pub fn sweep_aabb(&self, velocity: Vec3, other: &AABB) -> Option<SweepResult> {
        // Early exit if already overlapping
        if self.intersects(other) {
            return Some(SweepResult::new(0.0, Vec3::ZERO, self.center()));
        }

        // Minkowski sum - expand the static AABB by the moving AABB's size
        let my_size = self.size();
        let expanded = AABB {
            min: other.min - my_size * 0.5,
            max: other.max + my_size * 0.5,
        };

        // Ray-box intersection test from center of moving AABB
        let ray_origin = self.center();
        self.ray_box_intersection(ray_origin, velocity, &expanded)
    }

    /// Ray-box intersection test
    fn ray_box_intersection(&self, origin: Vec3, direction: Vec3, aabb: &AABB) -> Option<SweepResult> {
        if direction.length_squared() < 1e-8 {
            return None;
        }

        let inv_dir = Vec3::new(
            1.0 / direction.x,
            1.0 / direction.y,
            1.0 / direction.z,
        );

        let t1 = (aabb.min - origin) * inv_dir;
        let t2 = (aabb.max - origin) * inv_dir;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let t_near = tmin.max_element();
        let t_far = tmax.min_element();

        // No intersection if ray doesn't overlap the box
        if t_near > t_far || t_far < 0.0 {
            return None;
        }

        let t = if t_near < 0.0 { t_far } else { t_near };

        // Only care about intersections within [0, 1] (the velocity vector)
        if t < 0.0 || t > 1.0 {
            return None;
        }

        // Determine which face was hit by finding which component of tmin was largest
        let normal = if tmin.x > tmin.y && tmin.x > tmin.z {
            Vec3::new(-direction.x.signum(), 0.0, 0.0)
        } else if tmin.y > tmin.z {
            Vec3::new(0.0, -direction.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, -direction.z.signum())
        };

        let position = origin + direction * t;

        Some(SweepResult::new(t, normal, position))
    }

    /// Check collision with a voxel grid
    /// Returns all blocks that this AABB overlaps with
    pub fn get_overlapping_blocks(&self) -> Vec<WorldPos> {
        let mut blocks = Vec::new();

        // Get the integer bounds of blocks this AABB spans
        let min_x = self.min.x.floor() as i32;
        let min_y = self.min.y.floor() as i32;
        let min_z = self.min.z.floor() as i32;
        let max_x = self.max.x.ceil() as i32;
        let max_y = self.max.y.ceil() as i32;
        let max_z = self.max.z.ceil() as i32;

        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    blocks.push(WorldPos::new(x, y, z));
                }
            }
        }

        blocks
    }

    /// Sweep test against multiple blocks
    /// Returns the earliest collision
    pub fn sweep_blocks<F>(&self, velocity: Vec3, get_block: F) -> Option<SweepResult>
    where
        F: Fn(&WorldPos) -> BlockType,
    {
        // Get all blocks along the movement path
        let extended = AABB {
            min: self.min.min(self.min + velocity),
            max: self.max.max(self.max + velocity),
        };

        let blocks = extended.get_overlapping_blocks();
        let mut earliest: Option<SweepResult> = None;

        for block_pos in blocks {
            let block_type = get_block(&block_pos);

            // Skip non-solid blocks
            if !block_type.is_solid() {
                continue;
            }

            let block_aabb = AABB::from_block(&block_pos);

            if let Some(result) = self.sweep_aabb(velocity, &block_aabb) {
                match earliest {
                    None => earliest = Some(result),
                    Some(ref prev) if result.time < prev.time => earliest = Some(result),
                    _ => {}
                }
            }
        }

        earliest
    }

    /// Check if a point is inside this AABB
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }

    /// Get the penetration depth and normal if two AABBs are overlapping
    pub fn get_penetration(&self, other: &AABB) -> Option<(Vec3, f32)> {
        if !self.intersects(other) {
            return None;
        }

        // Calculate overlap on each axis
        let x_overlap = (self.max.x - other.min.x).min(other.max.x - self.min.x);
        let y_overlap = (self.max.y - other.min.y).min(other.max.y - self.min.y);
        let z_overlap = (self.max.z - other.min.z).min(other.max.z - self.min.z);

        // Find the axis with minimum penetration
        let min_overlap = x_overlap.min(y_overlap).min(z_overlap);

        let normal = if min_overlap == x_overlap {
            if self.center().x < other.center().x {
                Vec3::new(-1.0, 0.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            }
        } else if min_overlap == y_overlap {
            if self.center().y < other.center().y {
                Vec3::new(0.0, -1.0, 0.0)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            }
        } else {
            if self.center().z < other.center().z {
                Vec3::new(0.0, 0.0, -1.0)
            } else {
                Vec3::new(0.0, 0.0, 1.0)
            }
        };

        Some((normal, min_overlap))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_intersection() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b));

        let c = AABB::new(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(!aabb.contains_point(Vec3::splat(2.0)));
    }

    #[test]
    fn test_get_overlapping_blocks() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::splat(2.5));
        let blocks = aabb.get_overlapping_blocks();
        // Should cover 3x3x3 = 27 blocks
        assert_eq!(blocks.len(), 27);
    }
}
