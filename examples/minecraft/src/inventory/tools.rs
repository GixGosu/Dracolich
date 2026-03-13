use crate::types::BlockType;

/// Types of tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
}

/// Tool material tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ToolTier {
    Wood = 0,
    Stone = 1,
    Iron = 2,
    Diamond = 3,
}

/// Represents a tool with its properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tool {
    pub tool_type: ToolType,
    pub tier: ToolTier,
}

impl Tool {
    /// Creates a new tool
    pub fn new(tool_type: ToolType, tier: ToolTier) -> Self {
        Self { tool_type, tier }
    }

    /// Returns the maximum durability for this tool
    pub fn max_durability(&self) -> u32 {
        match self.tier {
            ToolTier::Wood => 60,
            ToolTier::Stone => 132,
            ToolTier::Iron => 251,
            ToolTier::Diamond => 1562,
        }
    }

    /// Returns the mining speed multiplier for this tool
    /// Base breaking time is multiplied by this value
    pub fn mining_speed(&self) -> f32 {
        match self.tier {
            ToolTier::Wood => 2.0,
            ToolTier::Stone => 4.0,
            ToolTier::Iron => 6.0,
            ToolTier::Diamond => 8.0,
        }
    }

    /// Returns true if this is the correct tool type for the given block
    pub fn is_correct_tool_for(&self, block: BlockType) -> bool {
        match self.tool_type {
            ToolType::Pickaxe => matches!(
                block,
                BlockType::Stone
                    | BlockType::Cobblestone
                    | BlockType::OreCoal
                    | BlockType::OreIron
                    | BlockType::OreGold
                    | BlockType::OreDiamond
            ),
            ToolType::Axe => matches!(
                block,
                BlockType::WoodOak | BlockType::WoodBirch | BlockType::Planks | BlockType::CraftingTable
            ),
            ToolType::Shovel => matches!(
                block,
                BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Gravel
            ),
        }
    }

    /// Returns true if this tool tier is sufficient to mine the given block
    /// Some blocks require a minimum tool tier to drop items
    pub fn can_harvest(&self, block: BlockType) -> bool {
        if !self.is_correct_tool_for(block) {
            return false;
        }

        match block {
            // Stone and coal ore can be mined with any pickaxe
            BlockType::Stone | BlockType::Cobblestone | BlockType::OreCoal => {
                self.tool_type == ToolType::Pickaxe
            },

            // Iron ore requires stone or better
            BlockType::OreIron => {
                self.tool_type == ToolType::Pickaxe && self.tier >= ToolTier::Stone
            },

            // Gold and diamond require iron or better
            BlockType::OreGold | BlockType::OreDiamond => {
                self.tool_type == ToolType::Pickaxe && self.tier >= ToolTier::Iron
            },

            // Wood blocks can be harvested with any axe
            BlockType::WoodOak | BlockType::WoodBirch | BlockType::Planks | BlockType::CraftingTable => {
                self.tool_type == ToolType::Axe
            },

            // Dirt, grass, sand, gravel can be harvested with any shovel
            BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Gravel => {
                self.tool_type == ToolType::Shovel
            },

            _ => false,
        }
    }

    /// Calculates the time (in seconds) to break a block with this tool
    pub fn break_time(&self, block: BlockType) -> f32 {
        let base_hardness = block.hardness();

        if self.is_correct_tool_for(block) {
            // Using correct tool - faster breaking
            base_hardness / self.mining_speed()
        } else if block.requires_tool() {
            // Block requires a tool but we're using the wrong one
            // Takes much longer
            base_hardness * 5.0
        } else {
            // Block doesn't require a tool (can break by hand)
            base_hardness * 1.5
        }
    }
}

impl BlockType {
    /// Returns true if this block requires a specific tool to harvest
    pub fn requires_tool(&self) -> bool {
        matches!(
            self,
            BlockType::Stone
                | BlockType::Cobblestone
                | BlockType::OreCoal
                | BlockType::OreIron
                | BlockType::OreGold
                | BlockType::OreDiamond
        )
    }

    /// Returns the minimum tool tier required to harvest this block
    /// Returns None if no tool is required
    pub fn minimum_tool_tier(&self) -> Option<ToolTier> {
        match self {
            BlockType::Stone | BlockType::Cobblestone | BlockType::OreCoal => {
                Some(ToolTier::Wood)
            },
            BlockType::OreIron => Some(ToolTier::Stone),
            BlockType::OreGold | BlockType::OreDiamond => {
                Some(ToolTier::Iron)
            },
            _ => None,
        }
    }

    /// Returns the preferred tool type for this block
    pub fn preferred_tool(&self) -> Option<ToolType> {
        match self {
            BlockType::Stone
            | BlockType::Cobblestone
            | BlockType::OreCoal
            | BlockType::OreIron
            | BlockType::OreGold
            | BlockType::OreDiamond => Some(ToolType::Pickaxe),

            BlockType::WoodOak | BlockType::WoodBirch | BlockType::Planks | BlockType::CraftingTable => {
                Some(ToolType::Axe)
            },

            BlockType::Dirt | BlockType::Grass | BlockType::Sand | BlockType::Gravel => {
                Some(ToolType::Shovel)
            },

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_durability() {
        assert_eq!(Tool::new(ToolType::Pickaxe, ToolTier::Wood).max_durability(), 60);
        assert_eq!(Tool::new(ToolType::Pickaxe, ToolTier::Stone).max_durability(), 132);
        assert_eq!(Tool::new(ToolType::Pickaxe, ToolTier::Diamond).max_durability(), 1562);
    }

    #[test]
    fn test_correct_tool() {
        let stone_pickaxe = Tool::new(ToolType::Pickaxe, ToolTier::Stone);
        assert!(stone_pickaxe.is_correct_tool_for(BlockType::Stone));
        assert!(stone_pickaxe.is_correct_tool_for(BlockType::OreIron));
        assert!(!stone_pickaxe.is_correct_tool_for(BlockType::WoodOak));
    }

    #[test]
    fn test_can_harvest() {
        let wood_pickaxe = Tool::new(ToolType::Pickaxe, ToolTier::Wood);
        let stone_pickaxe = Tool::new(ToolType::Pickaxe, ToolTier::Stone);

        assert!(wood_pickaxe.can_harvest(BlockType::OreCoal));
        assert!(!wood_pickaxe.can_harvest(BlockType::OreIron));

        assert!(stone_pickaxe.can_harvest(BlockType::OreIron));
        assert!(!stone_pickaxe.can_harvest(BlockType::OreDiamond));
    }

    #[test]
    fn test_mining_speed() {
        let wood_pick = Tool::new(ToolType::Pickaxe, ToolTier::Wood);
        let diamond_pick = Tool::new(ToolType::Pickaxe, ToolTier::Diamond);

        let wood_time = wood_pick.break_time(BlockType::Stone);
        let diamond_time = diamond_pick.break_time(BlockType::Stone);

        assert!(diamond_time < wood_time);
    }
}
