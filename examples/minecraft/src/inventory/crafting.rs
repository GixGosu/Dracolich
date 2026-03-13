use super::item::{Item, ItemStack};
use super::tools::{ToolTier, ToolType};
use crate::types::BlockType;

/// Represents a crafting grid (2x2 for inventory, 3x3 for crafting table)
#[derive(Debug, Clone)]
pub struct CraftingGrid {
    /// The grid size (2 or 3)
    pub size: usize,
    /// The slots in the grid (4 for 2x2, 9 for 3x3)
    pub slots: Vec<Option<ItemStack>>,
    /// The output slot
    pub output: Option<ItemStack>,
}

impl CraftingGrid {
    /// Creates a new 2x2 crafting grid (for inventory)
    pub fn new_2x2() -> Self {
        Self {
            size: 2,
            slots: vec![None; 4],
            output: None,
        }
    }

    /// Creates a new 3x3 crafting grid (for crafting table)
    pub fn new_3x3() -> Self {
        Self {
            size: 3,
            slots: vec![None; 9],
            output: None,
        }
    }

    /// Updates the output slot based on the current recipe
    pub fn update_output(&mut self) {
        self.output = self.match_recipe();
    }

    /// Attempts to match the current grid against all known recipes
    fn match_recipe(&self) -> Option<ItemStack> {
        for recipe in get_recipes().iter() {
            if let Some(output) = recipe.matches(self) {
                return Some(output);
            }
        }
        None
    }

    /// Takes the crafted item from the output slot
    /// Consumes one item from each input slot
    pub fn take_output(&mut self) -> Option<ItemStack> {
        if self.output.is_some() {
            // Consume ingredients
            for slot in &mut self.slots {
                if let Some(stack) = slot {
                    stack.count -= 1;
                    if stack.count == 0 {
                        *slot = None;
                    }
                }
            }

            let result = self.output.take();
            self.update_output();
            result
        } else {
            None
        }
    }

    /// Clears the crafting grid
    pub fn clear(&mut self) {
        self.slots.fill(None);
        self.output = None;
    }

    /// Sets an item in the grid
    pub fn set_slot(&mut self, index: usize, item: Option<ItemStack>) {
        if index < self.slots.len() {
            self.slots[index] = item;
            self.update_output();
        }
    }
}

/// Pattern for shaped recipes
#[derive(Debug, Clone, PartialEq)]
pub enum RecipePattern {
    /// Shapeless recipe - items can be in any position
    Shapeless(Vec<Item>),

    /// Shaped recipe - items must be in specific positions
    /// Pattern is row-major (top-left to bottom-right)
    /// None represents an empty slot
    Shaped2x2([Option<Item>; 4]),
    Shaped3x3([Option<Item>; 9]),
}

/// Represents a crafting recipe
#[derive(Debug, Clone)]
pub struct Recipe {
    pub pattern: RecipePattern,
    pub output: ItemStack,
}

impl Recipe {
    /// Checks if the crafting grid matches this recipe
    pub fn matches(&self, grid: &CraftingGrid) -> Option<ItemStack> {
        match &self.pattern {
            RecipePattern::Shapeless(items) => self.matches_shapeless(grid, items),
            RecipePattern::Shaped2x2(pattern) => self.matches_shaped_2x2(grid, pattern),
            RecipePattern::Shaped3x3(pattern) => self.matches_shaped_3x3(grid, pattern),
        }
    }

    fn matches_shapeless(&self, grid: &CraftingGrid, required_items: &[Item]) -> Option<ItemStack> {
        // Count items in the grid
        let mut grid_items: Vec<Item> = grid.slots
            .iter()
            .filter_map(|slot| slot.as_ref().map(|s| s.item))
            .collect();

        // Count required items
        let mut required = required_items.to_vec();

        // Must have same number of items
        if grid_items.len() != required.len() {
            return None;
        }

        // Sort both for comparison
        grid_items.sort_by_key(|item| format!("{:?}", item));
        required.sort_by_key(|item| format!("{:?}", item));

        if grid_items == required {
            Some(self.output)
        } else {
            None
        }
    }

    fn matches_shaped_2x2(&self, grid: &CraftingGrid, pattern: &[Option<Item>; 4]) -> Option<ItemStack> {
        if grid.size != 2 {
            return None;
        }

        for (i, slot) in grid.slots.iter().enumerate() {
            let grid_item = slot.as_ref().map(|s| s.item);
            if grid_item != pattern[i] {
                return None;
            }
        }

        Some(self.output)
    }

    fn matches_shaped_3x3(&self, grid: &CraftingGrid, pattern: &[Option<Item>; 9]) -> Option<ItemStack> {
        if grid.size != 3 {
            // Try to match in top-left corner of 3x3 grid
            return None;
        }

        for (i, slot) in grid.slots.iter().enumerate() {
            let grid_item = slot.as_ref().map(|s| s.item);
            if grid_item != pattern[i] {
                return None;
            }
        }

        Some(self.output)
    }
}

/// Get all available crafting recipes
/// Using WoodOak for logs and Planks for planks (matching BlockType enum)
pub fn get_recipes() -> Vec<Recipe> {
    vec![
        // Logs → 4 Planks (WoodOak → Planks)
        Recipe {
            pattern: RecipePattern::Shapeless(vec![
                Item::Block(BlockType::WoodOak),
            ]),
            output: ItemStack {
                item: Item::Block(BlockType::Planks),
                count: 4,
                durability: None,
            },
        },

        // Birch Logs → 4 Planks
        Recipe {
            pattern: RecipePattern::Shapeless(vec![
                Item::Block(BlockType::WoodBirch),
            ]),
            output: ItemStack {
                item: Item::Block(BlockType::Planks),
                count: 4,
                durability: None,
            },
        },

        // 4 Planks → Crafting Table
        Recipe {
            pattern: RecipePattern::Shaped2x2([
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
            ]),
            output: ItemStack {
                item: Item::Block(BlockType::CraftingTable),
                count: 1,
                durability: None,
            },
        },

        // 2 Planks Vertical → 4 Sticks
        Recipe {
            pattern: RecipePattern::Shaped2x2([
                None,
                None,
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
            ]),
            output: ItemStack {
                item: Item::Stick,
                count: 4,
                durability: None,
            },
        },

        // Alternative stick recipe (top row)
        Recipe {
            pattern: RecipePattern::Shaped2x2([
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                None,
                None,
            ]),
            output: ItemStack {
                item: Item::Stick,
                count: 4,
                durability: None,
            },
        },

        // Wooden Pickaxe
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                None,
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Pickaxe, ToolTier::Wood),
                count: 1,
                durability: Some(60),
            },
        },

        // Wooden Axe
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Block(BlockType::Planks)),
                None,
                Some(Item::Block(BlockType::Planks)),
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Axe, ToolTier::Wood),
                count: 1,
                durability: Some(60),
            },
        },

        // Wooden Shovel
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                None,
                Some(Item::Block(BlockType::Planks)),
                None,
                None,
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Shovel, ToolTier::Wood),
                count: 1,
                durability: Some(60),
            },
        },

        // Stone Pickaxe
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                None,
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Pickaxe, ToolTier::Stone),
                count: 1,
                durability: Some(132),
            },
        },

        // Stone Axe
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                None,
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Axe, ToolTier::Stone),
                count: 1,
                durability: Some(132),
            },
        },

        // Stone Shovel
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                None,
                Some(Item::Block(BlockType::Cobblestone)),
                None,
                None,
                Some(Item::Stick),
                None,
                None,
                Some(Item::Stick),
                None,
            ]),
            output: ItemStack {
                item: Item::Tool(ToolType::Shovel, ToolTier::Stone),
                count: 1,
                durability: Some(132),
            },
        },

        // Furnace (8 cobblestone in ring)
        Recipe {
            pattern: RecipePattern::Shaped3x3([
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                None,
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
                Some(Item::Block(BlockType::Cobblestone)),
            ]),
            output: ItemStack {
                item: Item::Block(BlockType::Furnace),
                count: 1,
                durability: None,
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_to_planks() {
        let mut grid = CraftingGrid::new_2x2();
        grid.slots[0] = Some(ItemStack::new(Item::Block(BlockType::WoodOak), 1));
        grid.update_output();

        assert!(grid.output.is_some());
        let output = grid.output.unwrap();
        assert_eq!(output.item, Item::Block(BlockType::Planks));
        assert_eq!(output.count, 4);
    }

    #[test]
    fn test_planks_to_sticks() {
        let mut grid = CraftingGrid::new_2x2();
        grid.slots[0] = Some(ItemStack::new(Item::Block(BlockType::Planks), 1));
        grid.slots[1] = Some(ItemStack::new(Item::Block(BlockType::Planks), 1));
        grid.update_output();

        assert!(grid.output.is_some());
        let output = grid.output.unwrap();
        assert_eq!(output.item, Item::Stick);
        assert_eq!(output.count, 4);
    }

    #[test]
    fn test_wooden_pickaxe() {
        let mut grid = CraftingGrid::new_3x3();
        grid.slots[0] = Some(ItemStack::new(Item::Block(BlockType::Planks), 1));
        grid.slots[1] = Some(ItemStack::new(Item::Block(BlockType::Planks), 1));
        grid.slots[2] = Some(ItemStack::new(Item::Block(BlockType::Planks), 1));
        grid.slots[4] = Some(ItemStack::new(Item::Stick, 1));
        grid.slots[7] = Some(ItemStack::new(Item::Stick, 1));
        grid.update_output();

        assert!(grid.output.is_some());
        let output = grid.output.unwrap();
        assert_eq!(output.item, Item::Tool(ToolType::Pickaxe, ToolTier::Wood));
        assert_eq!(output.count, 1);
    }
}
