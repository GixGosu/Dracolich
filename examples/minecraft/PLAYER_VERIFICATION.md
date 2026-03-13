# Player System - Verification Guide

This document provides verification steps to ensure the player system is working correctly.

## Pre-Integration Verification

### 1. Code Compilation

Run the following command to verify all player modules compile:

```bash
cargo build --lib
```

**Expected:** Clean compilation with no errors or warnings.

**If compilation fails:**
- Check for missing imports
- Verify all modules are properly declared in `mod.rs`
- Check that `src/lib.rs` exports the player module

### 2. Unit Tests

Run all player module tests:

```bash
cargo test --lib player
```

**Expected output:**
```
running 44 tests
test player::health::tests::test_damage ... ok
test player::health::tests::test_death ... ok
test player::health::tests::test_healing ... ok
...
test player::movement::tests::test_gravity ... ok
test player::movement::tests::test_jump ... ok
...
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured
```

**All 44 tests should pass:**
- mod.rs: 5 tests
- movement.rs: 9 tests
- health.rs: 10 tests
- interaction.rs: 8 tests
- hotbar.rs: 12 tests

### 3. Module Structure Check

Verify all files exist:

```bash
ls -la src/player/
```

**Expected files:**
- mod.rs (237 lines)
- movement.rs (246 lines)
- health.rs (240 lines)
- interaction.rs (329 lines)
- hotbar.rs (358 lines)

Total: ~1,410 lines of implementation code.

## Post-Integration Verification

After integrating the player system into the main game loop, verify these behaviors:

### Movement Tests

#### Test 1: Basic Walking
1. Launch game
2. Press W key
3. **Expected:** Player moves forward (negative Z direction by default)
4. Release W
5. **Expected:** Player stops quickly (velocity damping)

#### Test 2: Strafing
1. Press A key
2. **Expected:** Player moves left
3. Press D key
4. **Expected:** Player moves right
5. Press W+D simultaneously
6. **Expected:** Player moves diagonally at same speed as cardinal (no speed boost)

#### Test 3: Jumping
1. Ensure player is on ground
2. Press Space
3. **Expected:**
   - Player rises ~1.25 blocks
   - `on_ground` becomes false
   - Velocity.y starts at 8.5, decreases over time
4. Land on ground
5. **Expected:**
   - `on_ground` becomes true
   - Velocity.y resets to 0

#### Test 4: Sprinting
1. Press Shift + W
2. **Expected:** Player moves faster (~1.3× speed)
3. Release Shift
4. **Expected:** Player returns to walk speed

#### Test 5: Gravity
1. Walk off edge of block
2. **Expected:**
   - Player falls
   - Velocity.y decreases (becomes more negative)
   - Falls at accelerating rate up to terminal velocity

### Camera Tests

#### Test 6: Mouse Look Horizontal
1. Move mouse left
2. **Expected:** Camera rotates left (yaw decreases)
3. Move mouse right
4. **Expected:** Camera rotates right (yaw increases)
5. Rotate 360°
6. **Expected:** Returns to starting view (yaw wraps at 2π)

#### Test 7: Mouse Look Vertical
1. Move mouse down
2. **Expected:** Camera looks up (pitch increases)
3. Move mouse up
4. **Expected:** Camera looks down (pitch decreases)
5. Try to look straight up (90°)
6. **Expected:** Camera stops at 89° (pitch clamped)

### Collision Tests

#### Test 8: Floor Collision
1. Stand on ground
2. **Expected:** Player doesn't fall through floor
3. **Expected:** `on_ground` is true

#### Test 9: Wall Collision
1. Walk into a wall
2. **Expected:** Player stops, doesn't clip through
3. Move parallel to wall
4. **Expected:** Player slides along wall smoothly

#### Test 10: Ceiling Collision
1. Jump under a low ceiling (2 blocks high)
2. **Expected:**
   - Player head doesn't clip through ceiling
   - Upward velocity stops when hitting ceiling
   - Player falls back down

### Health Tests

#### Test 11: Fall Damage (Safe)
1. Jump from 2 blocks high
2. **Expected:** No damage taken (≤3 blocks is safe)

#### Test 12: Fall Damage (Dangerous)
1. Jump from 6 blocks high
2. **Expected:**
   - Take 3 damage (6 - 3 = 3)
   - Health reduces from 20 to 17
3. Jump from 23 blocks high
4. **Expected:** Instant death (20+ damage)

#### Test 13: Death and Respawn
1. Take lethal damage (fall from high place)
2. **Expected:**
   - `is_dead()` returns true
   - Health is 0
3. Trigger respawn
4. **Expected:**
   - Player teleports to spawn point
   - Health restored to 20
   - Velocity reset to zero

#### Test 14: Damage Cooldown
1. Take 5 damage
2. Immediately try to take more damage
3. **Expected:** No damage for 500ms (cooldown active)
4. Wait 500ms, take damage again
5. **Expected:** Damage applies

### Interaction Tests

#### Test 15: Block Targeting
1. Look at a block within 5 blocks
2. **Expected:**
   - `targeted_block` is Some
   - Block should be highlighted (if rendering implemented)
3. Look away or move >5 blocks away
4. **Expected:** `targeted_block` is None

#### Test 16: Block Breaking (Fast)
1. Target dirt block (0.5s break time)
2. Hold left mouse button
3. **Expected:**
   - Break progress increases from 0 to 1 over 0.5s
   - Block breaks and becomes Air
   - Break progress resets to 0

#### Test 17: Block Breaking (Slow)
1. Target stone block (4.0s break time)
2. Hold left mouse button
3. **Expected:**
   - Break progress increases slowly
   - After 4 seconds, block breaks
4. Start breaking another stone block
5. Release mouse button halfway
6. **Expected:**
   - Breaking cancels
   - Progress resets to 0

#### Test 18: Unbreakable Blocks
1. Target bedrock block
2. Hold left mouse button
3. **Expected:**
   - No breaking starts
   - Progress stays at 0

#### Test 19: Block Placement (Valid)
1. Select a block in hotbar (e.g., dirt)
2. Target a block within 5 blocks
3. Right-click on block face
4. **Expected:**
   - New block placed adjacent to targeted block
   - Block count decreases by 1 in hotbar slot
   - Block appears in world

#### Test 20: Block Placement (Invalid - Too Close)
1. Stand in a 1-block-tall space
2. Try to place block where player's head is
3. **Expected:**
   - Placement fails (would suffocate player)
   - No block placed

#### Test 21: Block Placement (Invalid - No Block)
1. Select empty hotbar slot
2. Right-click on block
3. **Expected:** Nothing happens (no block to place)

### Hotbar Tests

#### Test 22: Hotbar Selection (Number Keys)
1. Press 1 key
2. **Expected:** Slot 0 selected
3. Press 9 key
4. **Expected:** Slot 8 selected

#### Test 23: Hotbar Scrolling
1. Start at slot 0
2. Scroll mouse wheel down (or up, depends on direction)
3. **Expected:** Selection moves to slot 1
4. Continue scrolling through all 9 slots
5. **Expected:** Wraps around (slot 8 → slot 0)

#### Test 24: Item Consumption
1. Select slot with 10 dirt blocks
2. Place a dirt block
3. **Expected:** Count decreases to 9
4. Place 9 more blocks
5. **Expected:**
   - Count decreases to 0
   - Slot becomes empty
   - Can't place more (no blocks left)

#### Test 25: Hotbar Stacking
1. Start with 50 dirt in slot 0
2. Pick up 20 more dirt
3. **Expected:**
   - Stacks into slot 0 (becomes 64)
   - Remaining 6 go to next empty slot

## Debug Verification

Enable F3 debug mode and verify these values update correctly:

```
Position: (x, y, z)           # Should change when moving
Velocity: (vx, vy, vz)        # Should show movement speed
On Ground: true/false         # Should toggle when jumping
Health: 20/20                 # Should decrease when taking damage
Facing: North (-Z) (0.0°)     # Should change when looking around
Selected Slot: 0              # Should change with number keys/scroll
```

## Performance Verification

### Frame Rate Test
1. Run game with FPS counter visible
2. Stand still
3. **Expected:** Stable FPS (60+ for most systems)
4. Move around, jump, look around
5. **Expected:** FPS remains stable (player overhead is <0.1ms)

### Memory Leak Test
1. Play for 10 minutes
2. Check memory usage
3. **Expected:** Player memory footprint stays constant (~1KB)
4. No increase over time

## Common Issues

### Issue: Player falls through floor
**Cause:** World `get_block()` not returning solid blocks correctly
**Fix:** Verify world collision detection works

### Issue: Can't move at all
**Cause:** Input state not updating or physics disabled
**Fix:** Check that `apply_movement_input()` and `update_physics()` are called every frame

### Issue: Camera rotation inverted
**Cause:** Mouse delta sign incorrect
**Fix:** Negate dy in `update_look()` or in input processing

### Issue: Breaking blocks takes forever
**Cause:** Block hardness too high or tool multiplier too low
**Fix:** Check BlockType::hardness() values and tool_multiplier

### Issue: Can place blocks inside player
**Cause:** Collision check disabled or AABB intersection wrong
**Fix:** Verify `would_collide_with_block()` is working

### Issue: Health doesn't regenerate
**Cause:** No food/healing system implemented yet
**Fix:** This is expected behavior (needs to be added separately)

## Success Criteria

✅ All 44 unit tests pass
✅ Player can move in all directions smoothly
✅ Camera rotates correctly with mouse
✅ Player collides with blocks (no clipping)
✅ Gravity pulls player down
✅ Jumping works and reaches correct height
✅ Fall damage applies correctly
✅ Player can die and respawn
✅ Blocks can be broken with progress tracking
✅ Blocks can be placed at correct positions
✅ Hotbar selection works with keys and scroll
✅ Item counts decrease when placing blocks
✅ FPS remains stable (60+)
✅ No memory leaks

If all criteria are met, the player system is working correctly!

## Integration Checklist

Before marking player system as complete, verify:

- [ ] All source files created (5 files)
- [ ] All unit tests pass (44 tests)
- [ ] Module exported in src/lib.rs
- [ ] Documentation created (3 markdown files)
- [ ] Integration example provided
- [ ] Constants properly defined
- [ ] No compilation warnings
- [ ] Clean API with no tight coupling
- [ ] Physics integration tested
- [ ] World integration tested
- [ ] Input handling verified

## Next Agent Handoff

The player system is ready for integration. Next agent should:

1. Use `PLAYER_INTEGRATION_EXAMPLE.md` as starting point
2. Wire up player to main game loop
3. Connect input events to InputState
4. Implement camera rendering from player.eye_position()
5. Add block highlight rendering for targeted_block
6. Add break progress rendering (crack textures)
7. Add health bar UI using player.health.percentage()
8. Add hotbar UI rendering

All APIs are documented and ready to use.
