# Cyberpunk Roguelike Game

Build a browser-playable roguelike game with a cyberpunk aesthetic using Phaser 3.

## Core Requirements

### Technical Stack
- **Engine:** Phaser 3 (latest stable)
- **Language:** TypeScript
- **Build:** Vite for dev server and bundling
- **Target:** Modern browsers, single HTML file deployable

### Roguelike Elements
- Procedurally generated levels (grid-based dungeon/city floors)
- Permadeath (run ends on death, start fresh)
- Turn-based movement and combat
- Resource management (health, energy/ammo, credits)
- Item/weapon pickups with varying rarities
- Progressive difficulty across floors

### Cyberpunk Theme
- Neon color palette (cyan, magenta, electric blue, warning yellow on dark backgrounds)
- Tech-noir atmosphere: rain effects, flickering lights, CRT scanlines
- Setting: Megacorp tower infiltration or underground data heist
- Enemy types: Corporate security drones, cyborg guards, ICE programs, rogue AI
- Weapons: Plasma pistol, monoblade, EMP grenades, hacking tools
- Player character: Street samurai, netrunner, or tech-modded infiltrator

### Minimum Viable Features
1. **Movement:** Arrow keys or WASD, turn-based grid movement
2. **Combat:** Bump-to-attack melee, ranged weapons with ammo
3. **FOV:** Field of view / fog of war system
4. **Enemies:** At least 3 enemy types with different behaviors
5. **Items:** Health packs, ammo, at least 2 weapon types
6. **Levels:** Procedural generation, minimum 3 floors with stairs
7. **UI:** Health bar, floor indicator, inventory display, message log
8. **Win/Lose:** Reach the objective OR die trying

### Visual Style
- Tile-based graphics (can use colored rectangles/simple shapes if no sprites)
- Glow effects on neon elements
- Screen shake on damage
- Particle effects for gunfire/explosions
- Dark background with high-contrast UI

## Deliverables

1. **Playable game** in `output/[project]/game/` directory
2. **index.html** that loads and runs the game
3. **README.md** with:
   - How to run locally (`npm install && npm run dev`)
   - Game controls
   - Brief gameplay description
4. **Source code** organized in sensible modules (player, enemies, map, ui, etc.)

## Success Criteria

- Game loads in browser without errors
- Player can move through procedurally generated levels
- Combat works (player can attack enemies, enemies can damage player)
- Death restarts the game
- At least 3 floors are reachable
- Cyberpunk aesthetic is evident (colors, theme, enemy types)

## Constraints

- Single-session development (no external asset dependencies beyond Phaser CDN)
- If using sprites, use simple geometric shapes or Unicode/emoji as placeholders
- Keep scope achievable: a working simple game beats an ambitious broken one
