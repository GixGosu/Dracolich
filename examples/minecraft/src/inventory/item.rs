use crate::types::BlockType;

/// Represents all possible items in the game (blocks and tools)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    // Block items - can be placed in world
    Block(BlockType),

    // Tool items
    Tool(super::tools::ToolType, super::tools::ToolTier),

    // Crafting materials
    Stick,

    // Dropped items
    Coal,

    // Food items
    RawPorkchop,
    RottenFlesh,
}

impl Item {
    /// Returns the maximum stack size for this item
    pub fn max_stack_size(&self) -> u32 {
        match self {
            // Tools don't stack
            Item::Tool(_, _) => 1,
            // Everything else stacks to 64
            _ => 64,
        }
    }

    /// Returns true if this item is stackable with another
    pub fn can_stack_with(&self, other: &Item) -> bool {
        self == other && self.max_stack_size() > 1
    }

    /// Returns the display name for this item
    pub fn name(&self) -> &'static str {
        match self {
            Item::Block(block_type) => block_type.name(),
            Item::Tool(tool_type, tier) => {
                match (tool_type, tier) {
                    (super::tools::ToolType::Pickaxe, super::tools::ToolTier::Wood) => "Wooden Pickaxe",
                    (super::tools::ToolType::Pickaxe, super::tools::ToolTier::Stone) => "Stone Pickaxe",
                    (super::tools::ToolType::Pickaxe, super::tools::ToolTier::Iron) => "Iron Pickaxe",
                    (super::tools::ToolType::Pickaxe, super::tools::ToolTier::Diamond) => "Diamond Pickaxe",

                    (super::tools::ToolType::Axe, super::tools::ToolTier::Wood) => "Wooden Axe",
                    (super::tools::ToolType::Axe, super::tools::ToolTier::Stone) => "Stone Axe",
                    (super::tools::ToolType::Axe, super::tools::ToolTier::Iron) => "Iron Axe",
                    (super::tools::ToolType::Axe, super::tools::ToolTier::Diamond) => "Diamond Axe",

                    (super::tools::ToolType::Shovel, super::tools::ToolTier::Wood) => "Wooden Shovel",
                    (super::tools::ToolType::Shovel, super::tools::ToolTier::Stone) => "Stone Shovel",
                    (super::tools::ToolType::Shovel, super::tools::ToolTier::Iron) => "Iron Shovel",
                    (super::tools::ToolType::Shovel, super::tools::ToolTier::Diamond) => "Diamond Shovel",
                }
            },
            Item::Stick => "Stick",
            Item::Coal => "Coal",
            Item::RawPorkchop => "Raw Porkchop",
            Item::RottenFlesh => "Rotten Flesh",
        }
    }
}

/// Represents a stack of items with count and durability (for tools)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
    /// Durability remaining (only used for tools, None for blocks)
    pub durability: Option<u32>,
}

impl ItemStack {
    /// Creates a new item stack with the given item and count
    pub fn new(item: Item, count: u32) -> Self {
        let max_count = item.max_stack_size().min(count);
        let durability = match item {
            Item::Tool(tool_type, tier) => {
                Some(super::tools::Tool::new(tool_type, tier).max_durability())
            },
            _ => None,
        };

        Self {
            item,
            count: max_count,
            durability,
        }
    }

    /// Creates a single item stack
    pub fn single(item: Item) -> Self {
        Self::new(item, 1)
    }

    /// Returns true if this stack can merge with another
    pub fn can_merge_with(&self, other: &ItemStack) -> bool {
        self.item.can_stack_with(&other.item)
            && self.count < self.item.max_stack_size()
    }

    /// Attempts to merge another stack into this one
    /// Returns the number of items that couldn't fit
    pub fn merge(&mut self, other: &mut ItemStack) -> u32 {
        if !self.can_merge_with(other) {
            return other.count;
        }

        let max_stack = self.item.max_stack_size();
        let space_available = max_stack - self.count;
        let amount_to_transfer = space_available.min(other.count);

        self.count += amount_to_transfer;
        other.count -= amount_to_transfer;

        other.count
    }

    /// Splits this stack, returning a new stack with the specified count
    /// Returns None if count is invalid
    pub fn split(&mut self, count: u32) -> Option<ItemStack> {
        if count == 0 || count >= self.count {
            return None;
        }

        self.count -= count;
        Some(ItemStack {
            item: self.item,
            count,
            durability: self.durability, // Tools shouldn't split, but handle it anyway
        })
    }

    /// Damages a tool, returning true if it broke
    pub fn damage_tool(&mut self, amount: u32) -> bool {
        if let Some(ref mut durability) = self.durability {
            if *durability <= amount {
                *durability = 0;
                return true; // Tool broke
            }
            *durability -= amount;
        }
        false
    }

    /// Returns the durability percentage (0.0 to 1.0) for tools
    pub fn durability_percent(&self) -> Option<f32> {
        self.durability.map(|current| {
            if let Item::Tool(tool_type, tier) = self.item {
                let max = super::tools::Tool::new(tool_type, tier).max_durability();
                current as f32 / max as f32
            } else {
                1.0
            }
        })
    }
}

// Note: BlockType::name() is defined in types.rs
