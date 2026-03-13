// UI rendering system for Minecraft clone
// Handles all 2D UI elements with orthographic projection

pub mod text;
pub mod crosshair;
pub mod hotbar;
pub mod health;
pub mod debug;
pub mod inventory_screen;
pub mod pause_menu;

use glam::{Mat4, Vec3};
use crate::types::BlockType;
use crate::inventory::{Item, ToolType, ToolTier};

/// UI rendering state and coordinate system
pub struct UIRenderer {
    /// Orthographic projection matrix for UI (0,0 at top-left)
    projection: Mat4,
    /// Screen width in pixels
    screen_width: u32,
    /// Screen height in pixels
    screen_height: u32,

    // Sub-renderers
    pub text: text::TextRenderer,
    pub crosshair: crosshair::CrosshairRenderer,
    pub hotbar: hotbar::HotbarRenderer,
    pub health: health::HealthRenderer,
    pub debug: debug::DebugRenderer,
    pub inventory_screen: inventory_screen::InventoryScreenRenderer,
    pub pause_menu: pause_menu::PauseMenuRenderer,
}

impl UIRenderer {
    /// Create a new UI renderer for the given screen dimensions
    pub fn new(screen_width: u32, screen_height: u32) -> Result<Self, String> {
        let projection = Mat4::orthographic_rh(
            0.0, screen_width as f32,
            screen_height as f32, 0.0,
            -1.0, 1.0
        );

        // Initialize all sub-renderers
        let text = text::TextRenderer::new()?;
        let crosshair = crosshair::CrosshairRenderer::new()?;
        let hotbar = hotbar::HotbarRenderer::new()?;
        let health = health::HealthRenderer::new()?;
        let debug = debug::DebugRenderer::new()?;
        let inventory_screen = inventory_screen::InventoryScreenRenderer::new()?;
        let pause_menu = pause_menu::PauseMenuRenderer::new()?;

        Ok(Self {
            projection,
            screen_width,
            screen_height,
            text,
            crosshair,
            hotbar,
            health,
            debug,
            inventory_screen,
            pause_menu,
        })
    }

    /// Update screen size and projection matrix
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.projection = Mat4::orthographic_rh(
            0.0, width as f32,
            height as f32, 0.0,
            -1.0, 1.0
        );
    }

    /// Get the current projection matrix
    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

    /// Get screen dimensions
    pub fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    /// Render HUD (crosshair, hotbar, health) - rendered during gameplay
    pub fn render_hud(&mut self, context: &HUDContext) {
        // Enable alpha blending for UI
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }

        // Render in specific order (back to front)
        self.hotbar.render(&self.projection, &self.text, context.hotbar_state);
        self.health.render(&self.projection, &self.text, context.health, context.max_health);
        self.crosshair.render(&self.projection, self.screen_width, self.screen_height);

        // Debug overlay (F3)
        if context.show_debug {
            self.debug.render(&self.projection, &self.text, context.debug_info);
        }

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    /// Render full-screen UI (inventory, pause menu) - rendered over everything
    pub fn render_screen(&mut self, screen: &UIScreen) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }

        match screen {
            UIScreen::Inventory(state) => {
                self.inventory_screen.render(&self.projection, &self.text, state);
            }
            UIScreen::PauseMenu(state) => {
                self.pause_menu.render(&self.projection, &self.text, state);
            }
        }

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    /// Handle mouse click for interactive UI elements
    pub fn handle_click(&mut self, screen: &UIScreen, x: f32, y: f32) -> UIAction {
        match screen {
            UIScreen::Inventory(state) => {
                self.inventory_screen.handle_click(state, x, y)
            }
            UIScreen::PauseMenu(state) => {
                self.pause_menu.handle_click(state, x, y)
            }
        }
    }

    /// Handle mouse drag for inventory item movement
    pub fn handle_drag(&mut self, screen: &UIScreen, x: f32, y: f32) -> Option<DragAction> {
        match screen {
            UIScreen::Inventory(state) => {
                self.inventory_screen.handle_drag(state, x, y)
            }
            _ => None,
        }
    }
}

/// Context for HUD rendering (provided by game state)
#[derive(Clone)]
pub struct HUDContext<'a> {
    pub hotbar_state: &'a HotbarState,
    pub health: u32,
    pub max_health: u32,
    pub show_debug: bool,
    pub debug_info: &'a DebugInfo,
}

/// Hotbar rendering state
#[derive(Clone)]
pub struct HotbarState {
    pub selected_slot: usize,
    pub items: [Option<HotbarItem>; 9],
}

#[derive(Clone)]
pub struct HotbarItem {
    pub item_type: ItemType,
    pub count: u32,
    pub durability: Option<f32>, // 0.0 to 1.0 for tools
}

/// Debug overlay information
#[derive(Clone)]
pub struct DebugInfo {
    pub position: Vec3,
    pub chunk_coords: (i32, i32),
    pub facing: Vec3,
    pub fps: u32,
    pub loaded_chunks: usize,
}

/// Active UI screen overlay
pub enum UIScreen<'a> {
    Inventory(&'a InventoryState),
    PauseMenu(&'a PauseMenuState),
}

/// Inventory screen state
pub struct InventoryState {
    pub inventory_slots: [Option<InventoryItem>; 36],
    pub crafting_slots: [Option<InventoryItem>; 9],
    pub crafting_result: Option<InventoryItem>,
    pub held_item: Option<InventoryItem>,
}

#[derive(Clone)]
pub struct InventoryItem {
    pub item_type: ItemType,
    pub count: u32,
    pub durability: Option<f32>,
}

/// Item types for UI rendering
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    // Blocks
    Grass, Dirt, Stone, Cobblestone, Sand, Gravel,
    WoodOak, WoodBirch, Planks, LeavesOak, LeavesBirch,
    OreCoal, OreIron, OreGold, OreDiamond,
    Bedrock, Water, Glass, CraftingTable, Furnace,

    // Tools
    WoodenPickaxe, WoodenAxe, WoodenShovel,
    StonePickaxe, StoneAxe, StoneShovel,
    IronPickaxe, IronAxe, IronShovel,
    DiamondPickaxe, DiamondAxe, DiamondShovel,

    // Materials
    Stick, Coal, IronIngot, GoldIngot, Diamond,
}

/// Pause menu state
pub struct PauseMenuState {
    pub hovered_button: Option<PauseButton>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PauseButton {
    Resume,
    Quit,
}

/// Actions returned by UI interaction
pub enum UIAction {
    None,
    SlotClick { slot: usize },
    CraftingClick { slot: usize },
    TakeResult,
    PauseButton(PauseButton),
}

/// Drag actions for inventory
pub struct DragAction {
    pub from_slot: usize,
    pub to_slot: Option<usize>,
}

/// Convert BlockType to ItemType for UI rendering
impl From<BlockType> for ItemType {
    fn from(block: BlockType) -> Self {
        match block {
            BlockType::Air => ItemType::Dirt, // Fallback for Air (shouldn't be rendered)
            BlockType::Grass => ItemType::Grass,
            BlockType::Dirt => ItemType::Dirt,
            BlockType::Stone => ItemType::Stone,
            BlockType::Cobblestone => ItemType::Cobblestone,
            BlockType::Sand => ItemType::Sand,
            BlockType::Gravel => ItemType::Gravel,
            BlockType::Bedrock => ItemType::Bedrock,
            BlockType::WoodOak => ItemType::WoodOak,
            BlockType::WoodBirch => ItemType::WoodBirch,
            BlockType::LeavesOak => ItemType::LeavesOak,
            BlockType::LeavesBirch => ItemType::LeavesBirch,
            BlockType::Water => ItemType::Water,
            BlockType::Glass => ItemType::Glass,
            BlockType::OreCoal => ItemType::OreCoal,
            BlockType::OreIron => ItemType::OreIron,
            BlockType::OreGold => ItemType::OreGold,
            BlockType::OreDiamond => ItemType::OreDiamond,
            BlockType::Planks => ItemType::Planks,
            BlockType::CraftingTable => ItemType::CraftingTable,
            BlockType::Furnace => ItemType::Furnace,
        }
    }
}

/// Convert Item to ItemType for UI rendering
impl From<Item> for ItemType {
    fn from(item: Item) -> Self {
        match item {
            Item::Block(block_type) => block_type.into(),
            Item::Tool(tool_type, tier) => match (tool_type, tier) {
                (ToolType::Pickaxe, ToolTier::Wood) => ItemType::WoodenPickaxe,
                (ToolType::Axe, ToolTier::Wood) => ItemType::WoodenAxe,
                (ToolType::Shovel, ToolTier::Wood) => ItemType::WoodenShovel,
                (ToolType::Pickaxe, ToolTier::Stone) => ItemType::StonePickaxe,
                (ToolType::Axe, ToolTier::Stone) => ItemType::StoneAxe,
                (ToolType::Shovel, ToolTier::Stone) => ItemType::StoneShovel,
                (ToolType::Pickaxe, ToolTier::Iron) => ItemType::IronPickaxe,
                (ToolType::Axe, ToolTier::Iron) => ItemType::IronAxe,
                (ToolType::Shovel, ToolTier::Iron) => ItemType::IronShovel,
                (ToolType::Pickaxe, ToolTier::Diamond) => ItemType::DiamondPickaxe,
                (ToolType::Axe, ToolTier::Diamond) => ItemType::DiamondAxe,
                (ToolType::Shovel, ToolTier::Diamond) => ItemType::DiamondShovel,
            },
            Item::Stick => ItemType::Stick,
            Item::Coal => ItemType::Coal,
            Item::RawPorkchop => ItemType::Stick, // Placeholder - no porkchop icon defined
            Item::RottenFlesh => ItemType::Stick, // Placeholder - no rotten flesh icon defined
        }
    }
}
