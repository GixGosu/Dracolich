# Mob System Integration Guide

Quick start guide for integrating the mob system into the main game loop.

## Step 1: Add MobManager to GameState

```rust
use crate::mobs::MobManager;

pub struct GameState {
    pub world: World,
    pub player: Player,
    pub mob_manager: MobManager,  // ← Add this
    // ... other fields
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: World::new(12345),
            player: Player::new(),
            mob_manager: MobManager::new(),  // ← Initialize
            // ...
        }
    }
}
```

## Step 2: Update Mobs in Game Loop

```rust
impl GameState {
    pub fn update(&mut self, delta_time: f32) {
        // Update player
        self.player.update(delta_time, &self.world);

        // Update mobs (with world access)
        let is_night = self.world.is_night();
        let attack_events = self.mob_manager.update(
            delta_time,
            self.player.position,
            is_night,
            |pos| self.world.get_block(pos),
        );

        // Handle zombie attacks on player
        for attack in attack_events {
            self.player.take_damage(attack.damage);

            // Apply knockback
            let knockback_dir = (self.player.position - attack.attacker_position).normalize();
            self.player.velocity += knockback_dir * 2.0;
        }

        // Handle player attacking mobs
        if self.player.is_attacking() {
            let results = self.mob_manager.damage_mobs_in_range(
                self.player.position + self.player.get_facing_direction() * 1.5,
                2.0,  // Attack range
                5.0,  // Attack damage
            );

            for (_mob_idx, result) in results {
                if let CombatResult::Killed { drops } = result {
                    // TODO: Spawn item entities for drops
                    for drop in drops {
                        println!("Dropped {} x{}", drop.item.name(), drop.count);
                    }
                }
            }
        }
    }
}
```

## Step 3: Render Mobs

```rust
impl GameState {
    pub fn render(&self, renderer: &mut Renderer) {
        // Render world chunks
        renderer.render_world(&self.world);

        // Render mobs
        for mob in self.mob_manager.mobs() {
            if !mob.is_alive() {
                continue;
            }

            let color = mob.get_color();
            let vertices = mob.get_render_vertices();

            // Simple box rendering
            renderer.render_mob_box(
                &vertices,
                color,
            );
        }

        // Render player
        renderer.render_player(&self.player);
    }
}
```

## Step 4: Add Mob Box Rendering

Add to your renderer:

```rust
impl Renderer {
    pub fn render_mob_box(&mut self, vertices: &[[f32; 3]], color: [f32; 3]) {
        // Create indices for a box (12 triangles, 36 indices)
        let indices = [
            // Bottom face
            0, 1, 2, 0, 2, 3,
            // Top face
            4, 5, 6, 4, 6, 7,
            // Front face
            0, 1, 5, 0, 5, 4,
            // Back face
            2, 3, 7, 2, 7, 6,
            // Left face
            0, 3, 7, 0, 7, 4,
            // Right face
            1, 2, 6, 1, 6, 5,
        ];

        // Use your existing mesh rendering system
        self.render_colored_mesh(vertices, &indices, color);
    }
}
```

## Step 5: Add Day/Night Cycle (if not present)

```rust
impl World {
    pub fn is_night(&self) -> bool {
        // Simple time-based check
        // Assuming you have a time_of_day field (0.0 to 1.0)
        self.time_of_day > 0.5  // Night is second half of day
    }
}
```

## Step 6: Optional - Manual Mob Spawning (for testing)

```rust
// Spawn some test mobs near player
impl GameState {
    pub fn spawn_test_mobs(&mut self) {
        use crate::mobs::MobType;

        // Spawn a few pigs
        for i in 0..5 {
            let pos = self.player.position + Vec3::new(
                (i as f32) * 3.0,
                0.0,
                10.0,
            );
            self.mob_manager.spawn_mob(MobType::Pig, pos);
        }

        // Spawn a zombie
        let zombie_pos = self.player.position + Vec3::new(0.0, 0.0, 15.0);
        self.mob_manager.spawn_mob(MobType::Zombie, zombie_pos);
    }
}
```

## Step 7: Handle Item Drops (Optional)

If you have an item entity system:

```rust
pub struct ItemEntity {
    pub item: Item,
    pub count: u32,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl GameState {
    fn handle_mob_death_drops(&mut self, drops: Vec<ItemDrop>) {
        for drop in drops {
            self.item_entities.push(ItemEntity {
                item: drop.item,
                count: drop.count,
                position: drop.position,
                velocity: Vec3::new(0.0, 2.0, 0.0),  // Pop up slightly
            });
        }
    }
}
```

## Complete Example

```rust
// In your main game loop (src/game_loop.rs or similar)

use crate::mobs::{MobManager, MobType, CombatResult};

pub struct Game {
    world: World,
    player: Player,
    mob_manager: MobManager,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::new(12345),
            player: Player::new(Vec3::new(0.0, 70.0, 0.0)),
            mob_manager: MobManager::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update world
        self.world.update(delta_time);

        // Update player
        self.player.update(delta_time, &self.world);

        // Update mobs
        let attack_events = self.mob_manager.update(
            delta_time,
            self.player.position,
            self.world.is_night(),
            |pos| self.world.get_block(pos),
        );

        // Handle mob attacks
        for attack in attack_events {
            self.player.take_damage(attack.damage);
            let knockback = (self.player.position - attack.attacker_position).normalize() * 2.0;
            self.player.velocity += knockback;
        }

        // Handle player attacks
        if self.player.just_attacked() {
            let attack_pos = self.player.position + self.player.facing_direction() * 1.5;
            let results = self.mob_manager.damage_mobs_in_range(attack_pos, 2.0, 5.0);

            for (_idx, result) in results {
                if let CombatResult::Killed { drops } = result {
                    for drop in drops {
                        self.spawn_item_entity(drop);
                    }
                }
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        // World
        renderer.render_world(&self.world, &self.player.camera);

        // Mobs
        for mob in self.mob_manager.mobs() {
            if mob.is_alive() {
                renderer.render_mob_box(
                    &mob.get_render_vertices(),
                    mob.get_color(),
                );
            }
        }

        // Player
        renderer.render_player(&self.player);

        // UI
        renderer.render_ui(&self.player, &self.mob_manager);
    }

    fn spawn_item_entity(&mut self, drop: ItemDrop) {
        // TODO: Add to item entity list
        println!("Spawned drop: {} x{} at {:?}",
            drop.item.name(),
            drop.count,
            drop.position
        );
    }
}
```

## Debugging Tips

1. **No mobs spawning?**
   - Check `is_night` is working
   - Verify light level calculation
   - Try manual spawning first
   - Check spawn caps (max 20 pigs, 30 zombies)

2. **Mobs not moving?**
   - Ensure `delta_time` is reasonable (0.016 for 60 FPS)
   - Check ground detection is working
   - Verify `get_block()` closure is correct

3. **Zombies not attacking?**
   - Check detection range (16 blocks)
   - Verify player position is being passed correctly
   - Test attack range (1.5 blocks)

4. **Performance issues?**
   - Check mob count (keep under 50 for now)
   - Profile the update() call
   - Consider spatial partitioning if needed

## Configuration

Adjust spawn rates in `spawning.rs`:

```rust
// More pigs
SpawnConfig::passive().max_count = 40;

// More aggressive zombie spawning
let mut zombie_config = SpawnConfig::hostile();
zombie_config.spawn_rate = 2.0;  // 2 attempts per second
zombie_config.spawn_chance = 0.3;  // 30% success
```

## Testing

```bash
# Run mob system tests
cargo test --lib mobs

# Test specific mob
cargo test --lib mobs::pig

# Run with output
cargo test --lib mobs -- --nocapture
```

## Summary

Integration steps:
1. ✅ Add MobManager to game state
2. ✅ Call update() in game loop
3. ✅ Handle attack events
4. ✅ Render mob boxes
5. ✅ Connect to player damage system
6. ⚠️ Implement lighting for proper spawning
7. ⚠️ Add item entity system for drops

The mob system is fully functional and ready to use. Start with manual spawning for testing, then enable automatic spawning once lighting is implemented.
