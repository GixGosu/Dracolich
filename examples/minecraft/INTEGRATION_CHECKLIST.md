# Inventory System Integration Checklist

## Quick Start

The inventory and crafting system is complete and tested. Follow these steps to integrate it into the game.

## 1. Update Player State

**File:** `src/player/state.rs`

Add inventory fields:
```rust
use crate::inventory::{Inventory, CraftingGrid};

pub struct Player {
    // ... existing fields ...
    pub inventory: Inventory,
    pub crafting_grid: CraftingGrid,  // 2x2 always available
    pub crafting_table_grid: Option<CraftingGrid>,  // 3x3 when near table
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            // ... existing initialization ...
            inventory: Inventory::with_starting_items(),  // Or ::new() for empty
            crafting_grid: CraftingGrid::new_2x2(),
            crafting_table_grid: None,
        }
    }
}
```

## 2. Handle Hotbar Input

**File:** `src/player/controller.rs` or event handler

```rust
// Number keys 1-9
if key == Key::Num1 { player.inventory.select_slot(0); }
if key == Key::Num2 { player.inventory.select_slot(1); }
// ... up to 9

// Mouse wheel
if scroll_delta > 0.0 {
    player.inventory.select_next();
} else {
    player.inventory.select_previous();
}
```

## 3. Block Breaking Integration

**File:** `src/physics/raycast.rs` or block interaction handler

```rust
use crate::inventory::{Item, ItemStack, Tool};

// When player breaks a block:
fn break_block(player: &mut Player, block_pos: WorldPos, block_type: BlockType) {
    // Calculate break time based on tool
    let break_time = if let Some(stack) = player.inventory.selected_item() {
        match stack.item {
            Item::Tool(tool_type, tier) => {
                Tool::new(tool_type, tier).break_time(block_type)
            }
            _ => block_type.hardness() * 1.5  // Hand mining
        }
    } else {
        block_type.hardness() * 1.5
    };

    // ... wait for break_time seconds ...

    // Damage tool
    if let Some(stack) = player.inventory.selected_item_mut() {
        if let Item::Tool(_, _) = stack.item {
            if stack.damage_tool(1) {
                // Tool broke
                *player.inventory.selected_item_mut() = None;
            }
        }
    }

    // Remove block from world
    world.set_block(block_pos, BlockType::Air);

    // Drop item
    let dropped_item = get_dropped_item(block_type, player.inventory.selected_item());
    player.inventory.add_item(dropped_item);
}

// Determine what item drops
fn get_dropped_item(block: BlockType, tool: Option<&ItemStack>) -> ItemStack {
    // Check if player has correct tool
    let can_harvest = if let Some(stack) = tool {
        if let Item::Tool(tool_type, tier) = stack.item {
            Tool::new(tool_type, tier).can_harvest(block)
        } else {
            !block.requires_tool()
        }
    } else {
        !block.requires_tool()
    };

    if !can_harvest {
        return ItemStack::new(Item::Block(BlockType::Air), 0);  // Nothing drops
    }

    // Determine drop
    match block {
        BlockType::CoalOre => ItemStack::new(Item::Coal, 1),
        BlockType::Stone => ItemStack::new(Item::Block(BlockType::Cobblestone), 1),
        BlockType::OakLog => ItemStack::new(Item::Block(BlockType::OakLog), 1),
        BlockType::Grass => ItemStack::new(Item::Block(BlockType::Dirt), 1),
        _ => ItemStack::new(Item::Block(block), 1),
    }
}
```

## 4. Block Placement Integration

**File:** `src/physics/raycast.rs` or block interaction handler

```rust
// When player places a block (right click):
fn place_block(player: &mut Player, target_pos: WorldPos, face: Direction, world: &mut World) {
    // Check if player has a block in selected slot
    if let Some(stack) = player.inventory.selected_item() {
        if let Item::Block(block_type) = stack.item {
            // Calculate placement position (adjacent to target face)
            let place_pos = target_pos.offset(face);

            // Check if position is valid (not inside player, etc.)
            if !is_position_occupied(place_pos, player.position) {
                // Place block
                world.set_block(place_pos, block_type);

                // Remove from inventory
                if let Some(stack) = player.inventory.selected_item_mut() {
                    stack.count -= 1;
                    if stack.count == 0 {
                        *player.inventory.selected_item_mut() = None;
                    }
                }
            }
        }
    }
}
```

## 5. Crafting UI Integration

**File:** `src/ui/crafting.rs` or inventory UI handler

```rust
// When player opens crafting (in inventory or at table):
fn render_crafting_ui(player: &Player, is_table: bool) {
    let grid = if is_table && player.crafting_table_grid.is_some() {
        player.crafting_table_grid.as_ref().unwrap()
    } else {
        &player.crafting_grid
    };

    // Render grid slots
    for (i, slot) in grid.slots.iter().enumerate() {
        let pos = grid_slot_position(i, grid.size);
        if let Some(stack) = slot {
            render_item_icon(stack.item, pos);
            render_item_count(stack.count, pos);
        }
    }

    // Render output slot
    if let Some(output) = &grid.output {
        render_item_icon(output.item, output_position());
        render_item_count(output.count, output_position());
    }
}

// When player clicks crafting grid:
fn handle_crafting_click(player: &mut Player, slot: usize, is_table: bool) {
    let grid = if is_table && player.crafting_table_grid.is_some() {
        player.crafting_table_grid.as_mut().unwrap()
    } else {
        &mut player.crafting_grid
    };

    // Handle slot interactions
    // ... implement drag-and-drop logic ...

    // Always update output after grid changes
    grid.update_output();
}

// When player clicks output slot:
fn take_crafted_item(player: &mut Player, is_table: bool) {
    let grid = if is_table && player.crafting_table_grid.is_some() {
        player.crafting_table_grid.as_mut().unwrap()
    } else {
        &mut player.crafting_grid
    };

    if let Some(crafted) = grid.take_output() {
        player.inventory.add_item(crafted);
    }
}
```

## 6. Crafting Table Block Interaction

**File:** `src/world/blocks.rs` or interaction handler

```rust
// When player right-clicks crafting table:
fn interact_with_block(player: &mut Player, block_pos: WorldPos, block: BlockType) {
    match block {
        BlockType::CraftingTable => {
            // Enable 3x3 crafting
            player.crafting_table_grid = Some(CraftingGrid::new_3x3());
            // Open crafting UI
            open_crafting_ui(player, true);
        }
        _ => {}
    }
}

// When player closes crafting table UI:
fn close_crafting_table(player: &mut Player) {
    // Drop items from crafting grid back into inventory
    if let Some(grid) = player.crafting_table_grid.take() {
        for slot in grid.slots {
            if let Some(stack) = slot {
                player.inventory.add_item(stack);
            }
        }
    }
}
```

## 7. Render Hotbar

**File:** `src/ui/hud.rs`

```rust
fn render_hotbar(player: &Player) {
    for i in 0..9 {
        let slot_pos = hotbar_slot_position(i);

        // Render slot background
        render_slot_bg(slot_pos, i == player.inventory.selected_slot);

        // Render item
        if let Some(stack) = &player.inventory.slots[i] {
            render_item_icon(stack.item, slot_pos);

            // Render count (if > 1)
            if stack.count > 1 {
                render_text(stack.count.to_string(), slot_pos);
            }

            // Render durability bar for tools
            if let Some(percent) = stack.durability_percent() {
                render_durability_bar(percent, slot_pos);
            }
        }

        // Render slot number
        render_text((i + 1).to_string(), slot_pos);
    }
}
```

## 8. Render Full Inventory

**File:** `src/ui/inventory.rs`

```rust
fn render_inventory_screen(player: &Player) {
    // Render all 36 slots
    for (i, slot) in player.inventory.slots.iter().enumerate() {
        let is_hotbar = i < 9;
        let pos = inventory_slot_position(i, is_hotbar);

        render_slot_bg(pos, false);

        if let Some(stack) = slot {
            render_item_icon(stack.item, pos);

            if stack.count > 1 {
                render_text(stack.count.to_string(), pos);
            }

            if let Some(percent) = stack.durability_percent() {
                render_durability_bar(percent, pos);
            }
        }
    }

    // Render crafting grid
    render_crafting_ui(player, false);
}
```

## 9. Testing Integration

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_wood_to_pickaxe_workflow() {
        let mut player = Player::new(Vec3::ZERO);

        // Start with oak log
        player.inventory.add_item(ItemStack::new(Item::Block(BlockType::OakLog), 1));

        // Craft planks
        player.crafting_grid.set_slot(0, Some(ItemStack::single(Item::Block(BlockType::OakLog))));
        assert!(player.crafting_grid.output.is_some());
        let planks = player.crafting_grid.take_output().unwrap();
        assert_eq!(planks.count, 4);
        player.inventory.add_item(planks);

        // Craft sticks
        player.crafting_grid.set_slot(0, Some(ItemStack::single(Item::Block(BlockType::OakPlanks))));
        player.crafting_grid.set_slot(1, Some(ItemStack::single(Item::Block(BlockType::OakPlanks))));
        let sticks = player.crafting_grid.take_output().unwrap();
        assert_eq!(sticks.count, 4);
        player.inventory.add_item(sticks);

        // Should have 2 planks and 4 sticks remaining
        assert_eq!(player.inventory.count_item(Item::Block(BlockType::OakPlanks)), 2);
        assert_eq!(player.inventory.count_item(Item::Stick), 4);
    }
}
```

## 10. Common Patterns

### Check if player can mine block
```rust
let can_mine = if let Some(stack) = player.inventory.selected_item() {
    if let Item::Tool(tool_type, tier) = stack.item {
        Tool::new(tool_type, tier).can_harvest(block_type)
    } else {
        !block_type.requires_tool()
    }
} else {
    !block_type.requires_tool()
};
```

### Get mining speed multiplier
```rust
let speed = if let Some(stack) = player.inventory.selected_item() {
    if let Item::Tool(tool_type, tier) = stack.item {
        let tool = Tool::new(tool_type, tier);
        if tool.is_correct_tool_for(block_type) {
            tool.mining_speed()
        } else {
            0.5  // Wrong tool penalty
        }
    } else {
        1.0  // Hand mining
    }
} else {
    1.0
};
```

### Check if player has recipe ingredients
```rust
let has_ingredients =
    player.inventory.contains_item(Item::Block(BlockType::OakPlanks), 3) &&
    player.inventory.contains_item(Item::Stick, 2);
```

## Files Modified/Created

- ✅ `src/inventory/mod.rs` - Module exports
- ✅ `src/inventory/item.rs` - Item types and stacks
- ✅ `src/inventory/inventory.rs` - 36-slot inventory
- ✅ `src/inventory/crafting.rs` - Crafting grid and recipes
- ✅ `src/inventory/tools.rs` - Tool mechanics
- ⏳ `src/player/state.rs` - Add inventory fields
- ⏳ `src/player/controller.rs` - Hotbar input
- ⏳ `src/physics/raycast.rs` - Block breaking/placing
- ⏳ `src/ui/hud.rs` - Hotbar rendering
- ⏳ `src/ui/inventory.rs` - Inventory screen

## Verification Steps

1. Compile: `cargo build --release`
2. Run tests: `cargo test --lib inventory`
3. Launch game and verify:
   - [ ] Hotbar displays correctly
   - [ ] Scrolling changes selected slot
   - [ ] Breaking blocks adds items to inventory
   - [ ] Placing blocks consumes from inventory
   - [ ] Mining with tools is faster than hand
   - [ ] Tools lose durability and break
   - [ ] Can craft planks from logs
   - [ ] Can craft sticks from planks
   - [ ] Can craft tools with sticks + materials
   - [ ] Crafting table enables 3x3 recipes

## Performance Notes

- Inventory operations are O(36) worst case - negligible
- Recipe matching is O(13) - instant
- No heap allocations in hot paths
- All operations are copy-based (no expensive clones)

## Troubleshooting

**Items not stacking:**
- Check `can_stack_with()` - tools don't stack
- Verify item equality (Block variants must match exactly)

**Recipes not matching:**
- Use `grid.update_output()` after changing slots
- Check pattern orientation (shaped recipes are position-sensitive)
- Shapeless recipes require exact item counts

**Tool durability not working:**
- Call `damage_tool(1)` after each block break
- Check return value - true means tool broke
- Remove item from slot when broken

**Mining too slow/fast:**
- Verify `break_time()` calculation
- Check `is_correct_tool_for()` logic
- Ensure block hardness values are set in `types.rs`
