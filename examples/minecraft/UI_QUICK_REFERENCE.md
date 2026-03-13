# UI System - Quick Reference

Fast lookup guide for UI system APIs.

## Initialization

```rust
use crate::ui::UIRenderer;

let mut ui = UIRenderer::new(800, 600)?;
ui.resize(new_width, new_height); // On window resize
```

## HUD Rendering (Gameplay)

```rust
use crate::ui::{HUDContext, HotbarState, HotbarItem, DebugInfo};

let context = HUDContext {
    hotbar_state: &HotbarState {
        selected_slot: 3,
        items: [...], // [Option<HotbarItem>; 9]
    },
    health: 18,
    max_health: 20,
    show_debug: true,
    debug_info: &DebugInfo {
        position: Vec3::new(123.4, 64.0, -56.7),
        chunk_coords: (7, -4),
        facing: Vec3::new(0.0, 0.0, 1.0), // Forward vector
        fps: 60,
        loaded_chunks: 81,
    },
};

ui.render_hud(&context);
```

## Inventory Screen

```rust
use crate::ui::{UIScreen, InventoryState, InventoryItem};

let state = InventoryState {
    inventory_slots: [...], // [Option<InventoryItem>; 36]
    crafting_slots: [...],  // [Option<InventoryItem>; 9]
    crafting_result: Some(InventoryItem { ... }),
    held_item: None,
};

ui.render_screen(&UIScreen::Inventory(&state));
```

## Pause Menu

```rust
use crate::ui::{UIScreen, PauseMenuState, PauseButton};

let state = PauseMenuState {
    hovered_button: Some(PauseButton::Resume),
};

ui.render_screen(&UIScreen::PauseMenu(&state));
```

## Mouse Interaction

```rust
use crate::ui::UIAction;

// Inventory screen click
let action = ui.handle_click(&UIScreen::Inventory(&state), mouse_x, mouse_y);
match action {
    UIAction::SlotClick { slot } => { /* ... */ }
    UIAction::CraftingClick { slot } => { /* ... */ }
    UIAction::TakeResult => { /* ... */ }
    UIAction::None => {}
    _ => {}
}

// Pause menu click
let action = ui.handle_click(&UIScreen::PauseMenu(&state), mouse_x, mouse_y);
match action {
    UIAction::PauseButton(PauseButton::Resume) => { /* unpause */ }
    UIAction::PauseButton(PauseButton::Quit) => { /* exit */ }
    _ => {}
}
```

## Item Type Mapping

```rust
use crate::ui::ItemType;

let ui_item = match inventory_item {
    InvItemType::Grass => ItemType::Grass,
    InvItemType::WoodenPickaxe => ItemType::WoodenPickaxe,
    // ... etc
};
```

## Hotbar Item Creation

```rust
use crate::ui::HotbarItem;

let item = HotbarItem {
    item_type: ItemType::StonePickaxe,
    count: 1,
    durability: Some(0.75), // 75% durability, None for blocks
};
```

## Keyboard Input Handling

```rust
match key {
    VirtualKeyCode::F3 => show_debug = !show_debug,
    VirtualKeyCode::E => show_inventory = !show_inventory,
    VirtualKeyCode::Escape => {
        if show_inventory {
            show_inventory = false;
        } else {
            paused = !paused;
        }
    }
    VirtualKeyCode::Key1 => select_hotbar_slot(0),
    VirtualKeyCode::Key2 => select_hotbar_slot(1),
    // ... Key3-Key9
    _ => {}
}
```

## Text Rendering (Direct)

```rust
// If you need to render custom text
ui.text.render_text(
    &projection,
    "Hello World",
    x, y,          // Position
    1.5,           // Scale
    Vec4::new(1.0, 1.0, 1.0, 1.0), // Color (RGBA)
);

// Centered text
let width = ui.text.measure_text("Hello", 1.5);
let x = (screen_width - width) / 2.0;
ui.text.render_text(&projection, "Hello", x, y, 1.5, color);

// Or use helper
use crate::ui::text::render_centered;
render_centered(&ui.text, &projection, "Hello", center_x, y, 1.5, color);
```

## Crosshair Customization

```rust
// Change crosshair appearance
ui.crosshair.set_style(
    15.0,  // Line length
    3.0,   // Thickness
    6.0,   // Gap from center
);
```

## OpenGL State for UI

```rust
unsafe {
    // Before rendering UI
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    gl::Disable(gl::DEPTH_TEST);

    // Render UI here...

    // After rendering UI
    gl::Enable(gl::DEPTH_TEST);
}
```

## Common Patterns

### Game Loop Structure

```rust
loop {
    // Input
    handle_input(&mut game, &input);

    // Update
    if !game.paused && !game.show_inventory {
        player.update(dt);
        world.update(player.position);
    }

    // Render 3D
    renderer.begin_frame();
    world.render(&renderer, &camera);

    // Render UI
    if game.show_inventory {
        render_inventory(&mut ui, &player);
    } else if game.paused {
        render_pause_menu(&mut ui, &game);
    } else {
        render_hud(&mut ui, &player, &world, fps);
    }

    renderer.end_frame();
    window.swap_buffers();
}
```

### FPS Counter

```rust
struct FPSCounter {
    frame_times: VecDeque<f64>,
    last_time: Instant,
}

impl FPSCounter {
    fn tick(&mut self) {
        let delta = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();
        self.frame_times.push_back(delta);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
    }

    fn fps(&self) -> u32 {
        let avg = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        (1.0 / avg) as u32
    }
}
```

### Cardinal Direction Helper

```rust
// Already implemented in debug.rs
fn calculate_facing(facing: &Vec3) -> (&'static str, f32) {
    let yaw_rad = (-facing.x).atan2(facing.z);
    let degrees = yaw_rad.to_degrees();
    // Returns ("North", 0.0) or similar
}
```

## Type Reference

```rust
// Main types
pub struct UIRenderer { ... }
pub struct HUDContext<'a> { ... }
pub struct HotbarState { ... }
pub struct HotbarItem { ... }
pub struct DebugInfo { ... }
pub struct InventoryState { ... }
pub struct InventoryItem { ... }
pub struct PauseMenuState { ... }

// Enums
pub enum UIScreen<'a> {
    Inventory(&'a InventoryState),
    PauseMenu(&'a PauseMenuState),
}

pub enum UIAction {
    None,
    SlotClick { slot: usize },
    CraftingClick { slot: usize },
    TakeResult,
    PauseButton(PauseButton),
}

pub enum PauseButton {
    Resume,
    Quit,
}

pub enum ItemType {
    // Blocks
    Grass, Dirt, Stone, Cobblestone, Sand, Gravel,
    OakLog, OakPlanks, OakLeaves,
    CoalOre, IronOre, GoldOre, DiamondOre,
    Bedrock, Water,

    // Tools
    WoodenPickaxe, WoodenAxe, WoodenShovel,
    StonePickaxe, StoneAxe, StoneShovel,
    IronPickaxe, IronAxe, IronShovel,
    DiamondPickaxe, DiamondAxe, DiamondShovel,

    // Materials
    Stick, Coal, IronIngot, GoldIngot, Diamond,
}
```

## Constants

```rust
// Hotbar
SLOT_SIZE: 40.0
SLOT_SPACING: 4.0
HOTBAR_BOTTOM_MARGIN: 20.0

// Health
HEART_SIZE: 16.0
HEART_SPACING: 2.0

// Crosshair
Default size: 10.0
Default thickness: 2.0
Default gap: 4.0

// Debug overlay
DEBUG_MARGIN: 10.0
DEBUG_LINE_HEIGHT: 12.0

// Inventory screen
SLOT_SIZE: 36.0 (smaller than hotbar)
SLOT_SPACING: 4.0
GRID_MARGIN: 20.0

// Pause menu
BUTTON_WIDTH: 200.0
BUTTON_HEIGHT: 50.0
BUTTON_SPACING: 20.0
```

## File Locations

```
src/ui/
├── mod.rs                 # UIRenderer, types, enums
├── text.rs               # TextRenderer
├── crosshair.rs          # CrosshairRenderer
├── hotbar.rs             # HotbarRenderer
├── health.rs             # HealthRenderer
├── debug.rs              # DebugRenderer
├── inventory_screen.rs   # InventoryScreenRenderer
└── pause_menu.rs         # PauseMenuRenderer
```

## Common Errors & Solutions

**Error**: UI not visible
- Check that `render_hud()` or `render_screen()` is called after world rendering
- Verify depth testing is disabled for UI
- Ensure alpha blending is enabled

**Error**: Text appears as white squares
- This is expected - current implementation uses placeholder quads
- Font texture integration is a future enhancement

**Error**: Click detection not working
- Verify mouse coordinates are in screen space (0,0 = top-left)
- Check coordinate conversion if using different origin
- Ensure `handle_click()` receives correct x, y values

**Error**: Performance issues
- Profile to find bottleneck (likely text rendering)
- Consider reducing text scale
- Batch rendering where possible

## Tips

1. **Always disable depth testing** before rendering UI
2. **Enable alpha blending** for transparency
3. **Render UI last** (after 3D world)
4. **Hide cursor** during gameplay, show for menus
5. **Build UI state** from game state each frame (don't cache)
6. **Use orthographic projection** for pixel-perfect rendering

For complete documentation, see:
- **UI_SYSTEM.md** - Full technical reference
- **UI_INTEGRATION.md** - Step-by-step integration guide
- **UI_SUMMARY.md** - Implementation summary
