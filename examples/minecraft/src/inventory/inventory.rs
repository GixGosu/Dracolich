use super::item::{Item, ItemStack};

/// Player inventory with 36 slots (9 hotbar + 27 storage)
#[derive(Debug, Clone)]
pub struct Inventory {
    /// All inventory slots (0-8 are hotbar, 9-35 are storage)
    pub slots: [Option<ItemStack>; 36],

    /// Currently selected hotbar slot (0-8)
    pub selected_slot: usize,
}

impl Inventory {
    /// Creates a new empty inventory
    pub fn new() -> Self {
        Self {
            slots: [None; 36],
            selected_slot: 0,
        }
    }

    /// Creates an inventory with some starting items for testing
    pub fn with_starting_items() -> Self {
        let mut inventory = Self::new();

        // Give the player some starting items
        inventory.add_item(ItemStack::new(Item::Block(crate::types::BlockType::WoodOak), 16));
        inventory.add_item(ItemStack::new(Item::Block(crate::types::BlockType::Dirt), 32));
        inventory.add_item(ItemStack::new(Item::Block(crate::types::BlockType::Cobblestone), 24));

        inventory
    }

    /// Returns the currently selected item stack
    pub fn selected_item(&self) -> Option<&ItemStack> {
        self.slots[self.selected_slot].as_ref()
    }

    /// Returns a mutable reference to the currently selected item stack
    pub fn selected_item_mut(&mut self) -> &mut Option<ItemStack> {
        &mut self.slots[self.selected_slot]
    }

    /// Selects the next hotbar slot
    pub fn select_next(&mut self) {
        self.selected_slot = (self.selected_slot + 1) % 9;
    }

    /// Selects the previous hotbar slot
    pub fn select_previous(&mut self) {
        self.selected_slot = if self.selected_slot == 0 { 8 } else { self.selected_slot - 1 };
    }

    /// Selects a specific hotbar slot (0-8)
    pub fn select_slot(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_slot = slot;
        }
    }

    /// Attempts to add an item to the inventory
    /// Returns the number of items that couldn't fit
    pub fn add_item(&mut self, mut item_stack: ItemStack) -> u32 {
        // First pass: try to merge with existing stacks
        for slot in &mut self.slots {
            if let Some(existing) = slot {
                if existing.can_merge_with(&item_stack) {
                    let remaining = existing.merge(&mut item_stack);
                    if remaining == 0 {
                        return 0; // All items added
                    }
                }
            }
        }

        // Second pass: find empty slots
        for slot in &mut self.slots {
            if slot.is_none() && item_stack.count > 0 {
                *slot = Some(item_stack);
                return 0; // All items added
            }
        }

        // Return remaining count that couldn't fit
        item_stack.count
    }

    /// Removes a specific number of an item from the inventory
    /// Returns the actual number removed
    pub fn remove_item(&mut self, item: Item, count: u32) -> u32 {
        let mut remaining = count;

        for slot in &mut self.slots {
            if remaining == 0 {
                break;
            }

            if let Some(stack) = slot {
                if stack.item == item {
                    let to_remove = remaining.min(stack.count);
                    stack.count -= to_remove;
                    remaining -= to_remove;

                    if stack.count == 0 {
                        *slot = None;
                    }
                }
            }
        }

        count - remaining
    }

    /// Checks if the inventory contains at least the specified count of an item
    pub fn contains_item(&self, item: Item, count: u32) -> bool {
        let mut total = 0u32;

        for slot in &self.slots {
            if let Some(stack) = slot {
                if stack.item == item {
                    total += stack.count;
                    if total >= count {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Counts the total number of a specific item in the inventory
    pub fn count_item(&self, item: Item) -> u32 {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|stack| stack.item == item)
            .map(|stack| stack.count)
            .sum()
    }

    /// Swaps two inventory slots
    pub fn swap_slots(&mut self, slot_a: usize, slot_b: usize) {
        if slot_a < 36 && slot_b < 36 && slot_a != slot_b {
            self.slots.swap(slot_a, slot_b);
        }
    }

    /// Moves an item stack from one slot to another
    /// If the destination is occupied, attempts to merge or swap
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from >= 36 || to >= 36 || from == to {
            return;
        }

        // Get the items (we need to work around borrow checker)
        let from_item = self.slots[from].take();

        if let Some(mut from_stack) = from_item {
            if let Some(to_stack) = &mut self.slots[to] {
                // Try to merge
                if to_stack.can_merge_with(&from_stack) {
                    let remaining = to_stack.merge(&mut from_stack);
                    if remaining > 0 {
                        self.slots[from] = Some(from_stack);
                    }
                } else {
                    // Swap if they can't merge
                    let temp = self.slots[to].take();
                    self.slots[to] = Some(from_stack);
                    self.slots[from] = temp;
                }
            } else {
                // Destination is empty, just move
                self.slots[to] = Some(from_stack);
            }
        }
    }

    /// Splits a stack in a slot, placing half in the destination slot
    pub fn split_stack(&mut self, from: usize, to: usize) {
        if from >= 36 || to >= 36 || from == to {
            return;
        }

        if self.slots[to].is_some() {
            return; // Destination must be empty for splitting
        }

        if let Some(from_stack) = &mut self.slots[from] {
            let split_count = (from_stack.count + 1) / 2; // Round up
            if let Some(split_stack) = from_stack.split(split_count) {
                self.slots[to] = Some(split_stack);
            }
        }
    }

    /// Clears the entire inventory
    pub fn clear(&mut self) {
        self.slots = [None; 36];
        self.selected_slot = 0;
    }

    /// Returns an iterator over all non-empty slots with their indices
    pub fn occupied_slots(&self) -> impl Iterator<Item = (usize, &ItemStack)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.as_ref().map(|stack| (i, stack)))
    }

    /// Returns true if the inventory has space for at least one more item
    pub fn has_space(&self) -> bool {
        self.slots.iter().any(|slot| slot.is_none())
    }

    /// Returns the number of empty slots
    pub fn empty_slot_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_none()).count()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlockType;

    #[test]
    fn test_add_and_count() {
        let mut inv = Inventory::new();
        let dirt = Item::Block(BlockType::Dirt);

        inv.add_item(ItemStack::new(dirt, 32));
        assert_eq!(inv.count_item(dirt), 32);

        inv.add_item(ItemStack::new(dirt, 16));
        assert_eq!(inv.count_item(dirt), 48);
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new();
        let stone = Item::Block(BlockType::Stone);

        inv.add_item(ItemStack::new(stone, 64));
        let removed = inv.remove_item(stone, 10);

        assert_eq!(removed, 10);
        assert_eq!(inv.count_item(stone), 54);
    }

    #[test]
    fn test_swap_slots() {
        let mut inv = Inventory::new();
        inv.slots[0] = Some(ItemStack::new(Item::Block(BlockType::Dirt), 32));
        inv.slots[1] = Some(ItemStack::new(Item::Block(BlockType::Stone), 16));

        inv.swap_slots(0, 1);

        assert_eq!(inv.slots[0].unwrap().item, Item::Block(BlockType::Stone));
        assert_eq!(inv.slots[1].unwrap().item, Item::Block(BlockType::Dirt));
    }
}
