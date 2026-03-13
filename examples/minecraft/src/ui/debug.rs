// F3 Debug overlay
// Shows position, chunk coords, facing direction, FPS, loaded chunks

use super::{DebugInfo, text::TextRenderer};
use glam::{Mat4, Vec4};

const DEBUG_MARGIN: f32 = 10.0;
const DEBUG_LINE_HEIGHT: f32 = 12.0;
const DEBUG_TEXT_SCALE: f32 = 1.0;

/// Debug overlay renderer (F3)
pub struct DebugRenderer {
    // No GPU resources needed - just uses TextRenderer
}

impl DebugRenderer {
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    /// Render debug overlay in top-left corner
    pub fn render(&self, projection: &Mat4, text: &TextRenderer, info: &DebugInfo) {
        let x = DEBUG_MARGIN;
        let mut y = DEBUG_MARGIN;

        let text_color = Vec4::new(1.0, 1.0, 1.0, 0.9);

        // Title
        text.render_text(
            projection,
            "Debug Info (F3 to toggle)",
            x,
            y,
            DEBUG_TEXT_SCALE,
            Vec4::new(1.0, 1.0, 0.0, 1.0),
        );
        y += DEBUG_LINE_HEIGHT * 1.5;

        // FPS
        let fps_text = format!("FPS: {}", info.fps);
        text.render_text(projection, &fps_text, x, y, DEBUG_TEXT_SCALE, text_color);
        y += DEBUG_LINE_HEIGHT;

        // Position
        let pos_text = format!(
            "XYZ: {:.2} / {:.2} / {:.2}",
            info.position.x, info.position.y, info.position.z
        );
        text.render_text(projection, &pos_text, x, y, DEBUG_TEXT_SCALE, text_color);
        y += DEBUG_LINE_HEIGHT;

        // Chunk coordinates
        let chunk_text = format!("Chunk: {} {}", info.chunk_coords.0, info.chunk_coords.1);
        text.render_text(projection, &chunk_text, x, y, DEBUG_TEXT_SCALE, text_color);
        y += DEBUG_LINE_HEIGHT;

        // Facing direction with compass
        let (direction, degrees) = calculate_facing(&info.facing);
        let facing_text = format!("Facing: {} ({:.1}°)", direction, degrees);
        text.render_text(projection, &facing_text, x, y, DEBUG_TEXT_SCALE, text_color);
        y += DEBUG_LINE_HEIGHT;

        // Loaded chunks
        let chunks_text = format!("Loaded chunks: {}", info.loaded_chunks);
        text.render_text(projection, &chunks_text, x, y, DEBUG_TEXT_SCALE, text_color);
        y += DEBUG_LINE_HEIGHT;

        // Additional spacing
        y += DEBUG_LINE_HEIGHT * 0.5;

        // Block looking at (placeholder - would need raycasting integration)
        let looking_text = "Looking at: Air";
        text.render_text(
            projection,
            looking_text,
            x,
            y,
            DEBUG_TEXT_SCALE,
            Vec4::new(0.8, 0.8, 0.8, 0.9),
        );
    }
}

/// Calculate cardinal direction and degrees from facing vector
fn calculate_facing(facing: &glam::Vec3) -> (&'static str, f32) {
    // Calculate yaw angle from facing vector (XZ plane)
    let yaw_rad = (-facing.x).atan2(facing.z);
    let mut degrees = yaw_rad.to_degrees();

    // Normalize to 0-360
    if degrees < 0.0 {
        degrees += 360.0;
    }

    // Determine cardinal direction
    let direction = if degrees >= 337.5 || degrees < 22.5 {
        "North"
    } else if degrees >= 22.5 && degrees < 67.5 {
        "Northeast"
    } else if degrees >= 67.5 && degrees < 112.5 {
        "East"
    } else if degrees >= 112.5 && degrees < 157.5 {
        "Southeast"
    } else if degrees >= 157.5 && degrees < 202.5 {
        "South"
    } else if degrees >= 202.5 && degrees < 247.5 {
        "Southwest"
    } else if degrees >= 247.5 && degrees < 292.5 {
        "West"
    } else {
        "Northwest"
    };

    (direction, degrees)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_facing_directions() {
        // North (0°)
        let (dir, deg) = calculate_facing(&Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(dir, "North");
        assert!((deg - 0.0).abs() < 1.0);

        // East (90°)
        let (dir, deg) = calculate_facing(&Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(dir, "East");
        assert!((deg - 90.0).abs() < 1.0);

        // South (180°)
        let (dir, deg) = calculate_facing(&Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(dir, "South");
        assert!((deg - 180.0).abs() < 1.0);

        // West (270°)
        let (dir, deg) = calculate_facing(&Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(dir, "West");
        assert!((deg - 270.0).abs() < 1.0);
    }
}
