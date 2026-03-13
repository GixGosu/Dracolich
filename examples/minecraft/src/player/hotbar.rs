// Player hotbar inventory
// 9-slot quick access bar for blocks and items

use crate::types::BlockType;

/// Number of slots in the hotbar
pub const HOTBAR_SIZE: usize = 9;

/// A single slot in the hotbar
#[derive(Debug, Clone, Copy)]
pub struct HotbarSlot {
    pub block_type: Option<BlockType>,
    pub count: u32,
}

impl HotbarSlot {
    pub fn empty() -> Self {
        Self {
            block_type: None,
            count: 0,
        }
    }

    pub fn new(block_type: BlockType, count: u32) -> Self {
        Self {
            block_type: Some(block_type),
            count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.block_type.is_none() || self.count == 0
    }

    pub fn add(&mut self, amount: u32) -> u32 {
        const MAX_STACK_SIZE: u32 = 64;

        let available = MAX_STACK_SIZE - self.count;
        let to_add = amount.min(available);
        self.count += to_add;

        amount - to_add // Return overflow
    }

    pub fn remove(&mut self, amount: u32) -> u32 {
        let removed = amount.min(self.count);
        self.count -= removed;

        if self.count == 0 {
            self.block_type = None;
        }

        removed
    }
}

/// 9-slot hotbar for quick item access
pub struct Hotbar {
    slots: [HotbarSlot; HOTBAR_SIZE],
}

impl Hotbar {
    /// Create a new empty hotbar
    pub fn new() -> Self {
        Self {
            slots: [HotbarSlot::empty(); HOTBAR_SIZE],
        }
    }

    /// Create a hotbar with some starting items (for testing/creative mode)
    pub fn with_starting_items() -> Self {
        let mut hotbar = Self::new();

        hotbar.set_slot(0, BlockType::Grass, 64);
        hotbar.set_slot(1, BlockType::Dirt, 64);
        hotbar.set_slot(2, BlockType::Stone, 64);
        hotbar.set_slot(3, BlockType::Cobblestone, 64);
        hotbar.set_slot(4, BlockType::Planks, 64);
        hotbar.set_slot(5, BlockType::WoodOak, 64);
        hotbar.set_slot(6, BlockType::Glass, 64);
        hotbar.set_slot(7, BlockType::Sand, 64);
        hotbar.set_slot(8, BlockType::LeavesOak, 64);

        hotbar
    }

    /// Get the block type in a slot
    pub fn get_block(&self, slot: usize) -> Option<BlockType> {
        if slot >= HOTBAR_SIZE {
            return None;
        }
        self.slots[slot].block_type
    }

    /// Get the count in a slot
    pub fn get_count(&self, slot: usize) -> u32 {
        if slot >= HOTBAR_SIZE {
            return 0;
        }
        self.slots[slot].count
    }

    /// Get a reference to a slot
    pub fn get_slot(&self, slot: usize) -> Option<&HotbarSlot> {
        self.slots.get(slot)
    }

    /// Get a mutable reference to a slot
    pub fn get_slot_mut(&mut self, slot: usize) -> Option<&mut HotbarSlot> {
        self.slots.get_mut(slot)
    }

    /// Set a slot to a specific block type and count
    pub fn set_slot(&mut self, slot: usize, block_type: BlockType, count: u32) {
        if slot >= HOTBAR_SIZE {
            return;
        }

        self.slots[slot] = HotbarSlot::new(block_type, count);
    }

    /// Clear a slot
    pub fn clear_slot(&mut self, slot: usize) {
        if slot >= HOTBAR_SIZE {
            return;
        }

        self.slots[slot] = HotbarSlot::empty();
    }

    /// Add items to the hotbar
    /// Returns the number of items that couldn't be added (overflow)
    pub fn add_item(&mut self, block_type: BlockType, mut count: u32) -> u32 {
        // First, try to add to existing stacks of the same type
        for slot in &mut self.slots {
            if slot.block_type == Some(block_type) && count > 0 {
                count = slot.add(count);
            }
        }

        // Then, try to fill empty slots
        if count > 0 {
            for slot in &mut self.slots {
                if slot.is_empty() {
                    *slot = HotbarSlot::new(block_type, 0);
                    count = slot.add(count);
                    if count == 0 {
                        break;
                    }
                }
            }
        }

        count // Return any remaining items that didn't fit
    }

    /// Remove items from the hotbar
    /// Returns the number of items actually removed
    pub fn remove_item(&mut self, block_type: BlockType, mut count: u32) -> u32 {
        let mut removed_total = 0;

        for slot in &mut self.slots {
            if slot.block_type == Some(block_type) && count > 0 {
                let removed = slot.remove(count);
                removed_total += removed;
                count -= removed;
            }
        }

        removed_total
    }

    /// Remove one item from a specific slot
    /// Returns true if an item was removed
    pub fn consume_from_slot(&mut self, slot: usize) -> bool {
        if slot >= HOTBAR_SIZE {
            return false;
        }

        if self.slots[slot].count > 0 {
            self.slots[slot].remove(1);
            true
        } else {
            false
        }
    }

    /// Check if the hotbar has at least one of a specific block type
    pub fn has_item(&self, block_type: BlockType) -> bool {
        self.slots.iter().any(|slot| slot.block_type == Some(block_type) && slot.count > 0)
    }

    /// Count total items of a specific type
    pub fn count_item(&self, block_type: BlockType) -> u32 {
        self.slots
            .iter()
            .filter(|slot| slot.block_type == Some(block_type))
            .map(|slot| slot.count)
            .sum()
    }

    /// Get all slots as a slice
    pub fn slots(&self) -> &[HotbarSlot; HOTBAR_SIZE] {
        &self.slots
    }

    /// Swap two slots
    pub fn swap_slots(&mut self, slot_a: usize, slot_b: usize) {
        if slot_a >= HOTBAR_SIZE || slot_b >= HOTBAR_SIZE {
            return;
        }

        self.slots.swap(slot_a, slot_b);
    }
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotbar_creation() {
        let hotbar = Hotbar::new();
        for i in 0..HOTBAR_SIZE {
            assert!(hotbar.get_block(i).is_none());
            assert_eq!(hotbar.get_count(i), 0);
        }
    }

    #[test]
    fn test_set_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.set_slot(0, BlockType::Dirt, 32);

        assert_eq!(hotbar.get_block(0), Some(BlockType::Dirt));
        assert_eq!(hotbar.get_count(0), 32);
    }

    #[test]
    fn test_clear_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.set_slot(0, BlockType::Stone, 16);
        hotbar.clear_slot(0);

        assert!(hotbar.get_block(0).is_none());
        assert_eq!(hotbar.get_count(0), 0);
    }

    #[test]
    fn test_add_item_to_empty() {
        let mut hotbar = Hotbar::new();

        let overflow = hotbar.add_item(BlockType::Cobblestone, 50);
        assert_eq!(overflow, 0);
        assert_eq!(hotbar.count_item(BlockType::Cobblestone), 50);
    }

    #[test]
    fn test_add_item_stacking() {
        let mut hotbar = Hotbar::new();

        hotbar.add_item(BlockType::Dirt, 32);
        hotbar.add_item(BlockType::Dirt, 32);

        // Should stack into same slot
        assert_eq!(hotbar.count_item(BlockType::Dirt), 64);
    }

    #[test]
    fn test_add_item_overflow() {
        let mut hotbar = Hotbar::new();

        // Fill all 9 slots with max stacks (64 each)
        for _ in 0..9 {
            hotbar.add_item(BlockType::Stone, 64);
        }

        // Try to add more - should overflow
        let overflow = hotbar.add_item(BlockType::Stone, 10);
        assert_eq!(overflow, 10);
        assert_eq!(hotbar.count_item(BlockType::Stone), 64 * 9);
    }

    #[test]
    fn test_remove_item() {
        let mut hotbar = Hotbar::new();

        hotbar.add_item(BlockType::Planks, 100);
        let removed = hotbar.remove_item(BlockType::Planks, 30);

        assert_eq!(removed, 30);
        assert_eq!(hotbar.count_item(BlockType::Planks), 70);
    }

    #[test]
    fn test_remove_more_than_available() {
        let mut hotbar = Hotbar::new();

        hotbar.add_item(BlockType::Glass, 20);
        let removed = hotbar.remove_item(BlockType::Glass, 50);

        assert_eq!(removed, 20);
        assert_eq!(hotbar.count_item(BlockType::Glass), 0);
    }

    #[test]
    fn test_consume_from_slot() {
        let mut hotbar = Hotbar::new();
        hotbar.set_slot(3, BlockType::Sand, 5);

        assert!(hotbar.consume_from_slot(3));
        assert_eq!(hotbar.get_count(3), 4);

        // Consume all remaining
        for _ in 0..4 {
            assert!(hotbar.consume_from_slot(3));
        }

        // Should be empty now
        assert!(!hotbar.consume_from_slot(3));
        assert!(hotbar.get_block(3).is_none());
    }

    #[test]
    fn test_has_item() {
        let mut hotbar = Hotbar::new();

        assert!(!hotbar.has_item(BlockType::WoodOak));

        hotbar.add_item(BlockType::WoodOak, 1);
        assert!(hotbar.has_item(BlockType::WoodOak));
    }

    #[test]
    fn test_swap_slots() {
        let mut hotbar = Hotbar::new();
        hotbar.set_slot(0, BlockType::Dirt, 10);
        hotbar.set_slot(5, BlockType::Stone, 20);

        hotbar.swap_slots(0, 5);

        assert_eq!(hotbar.get_block(0), Some(BlockType::Stone));
        assert_eq!(hotbar.get_count(0), 20);
        assert_eq!(hotbar.get_block(5), Some(BlockType::Dirt));
        assert_eq!(hotbar.get_count(5), 10);
    }

    #[test]
    fn test_starting_items() {
        let hotbar = Hotbar::with_starting_items();

        // Should have items in all 9 slots
        for i in 0..HOTBAR_SIZE {
            assert!(hotbar.get_block(i).is_some());
            assert_eq!(hotbar.get_count(i), 64);
        }
    }

    #[test]
    fn test_hotbar_slot_add() {
        let mut slot = HotbarSlot::new(BlockType::Grass, 60);

        // Add 4 (within limit)
        let overflow = slot.add(4);
        assert_eq!(overflow, 0);
        assert_eq!(slot.count, 64);

        // Try to add more (should overflow)
        let overflow = slot.add(10);
        assert_eq!(overflow, 10);
        assert_eq!(slot.count, 64);
    }

    #[test]
    fn test_hotbar_slot_remove() {
        let mut slot = HotbarSlot::new(BlockType::Cobblestone, 30);

        let removed = slot.remove(10);
        assert_eq!(removed, 10);
        assert_eq!(slot.count, 20);

        // Remove all remaining
        let removed = slot.remove(100);
        assert_eq!(removed, 20);
        assert_eq!(slot.count, 0);
        assert!(slot.is_empty());
    }
}
