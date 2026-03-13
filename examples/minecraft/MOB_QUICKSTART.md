# Mob System Quick Start

30-second integration guide.

## Add to GameState

```rust
use crate::mobs::MobManager;

pub struct GameState {
    pub mob_manager: MobManager,
    // ... other fields
}
```

## Update Loop

```rust
// Update mobs
let attacks = self.mob_manager.update(
    delta_time,
    player.position,
    world.is_night(),
    |pos| world.get_block(pos),
);

// Handle zombie attacks
for attack in attacks {
    player.take_damage(attack.damage);
}
```

## Render

```rust
for mob in self.mob_manager.mobs() {
    if mob.is_alive() {
        renderer.render_box(
            &mob.get_render_vertices(),
            mob.get_color(),
        );
    }
}
```

## Test Spawning

```rust
use crate::mobs::MobType;

// Spawn a pig
mob_manager.spawn_mob(MobType::Pig, Vec3::new(10.0, 65.0, 10.0));

// Spawn a zombie
mob_manager.spawn_mob(MobType::Zombie, Vec3::new(-10.0, 65.0, -10.0));
```

## Stats

**Pig:**
- Health: 10
- Speed: 2.0
- Behavior: Wanders randomly
- Drops: Raw Porkchop (1-3)
- Color: Pink [1.0, 0.75, 0.8]

**Zombie:**
- Health: 20
- Speed: 3.5
- Damage: 3 (1.5 block range)
- Behavior: Chases player, attacks
- Drops: Rotten Flesh (0-2)
- Color: Green [0.3, 0.8, 0.3]

## Full Docs

- **MOB_SYSTEM.md** - Technical details
- **MOB_INTEGRATION_GUIDE.md** - Complete integration
- **MOB_SYSTEM_SUMMARY.md** - Summary and handoff
