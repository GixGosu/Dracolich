# Physics Engine Verification Guide

This document describes how to verify that the physics engine is working correctly once the game is compiled and running.

## Compilation Verification

First, ensure the code compiles:

```bash
cd /mnt/e/Dev/Draco/output/2026-03-12T23-15-05-build-minecraft-from-scratch
cargo build --release
```

Expected: Clean compilation with no errors.

## Unit Tests

Run the included unit tests:

```bash
cargo test --release
```

Expected tests:
- `physics::aabb::tests::test_aabb_intersection` ✓
- `physics::aabb::tests::test_contains_point` ✓
- `physics::aabb::tests::test_get_overlapping_blocks` ✓
- `physics::collision::tests::test_free_movement` ✓
- `physics::collision::tests::test_ground_collision` ✓
- `physics::collision::tests::test_is_on_ground` ✓
- `physics::raycast::tests::test_raycast_hit` ✓
- `physics::raycast::tests::test_raycast_miss` ✓
- `physics::raycast::tests::test_line_of_sight` ✓

All tests should pass.

## Runtime Verification

Once the game is running, verify each physics feature:

### 1. Gravity and Falling ✓

**Test:** Jump or walk off a ledge

**Expected behavior:**
- Player accelerates downward at 32 blocks/s²
- Falls smoothly without stuttering
- Reaches terminal velocity after a long fall
- Lands softly on ground below

**Failure cases to check:**
- ❌ Player clips through floor when landing
- ❌ Player falls at inconsistent speed
- ❌ Player can walk on air

### 2. Ground Collision ✓

**Test:** Walk on solid ground

**Expected behavior:**
- Player walks smoothly on all solid blocks
- No bouncing or jittering
- Transitions smoothly between different block types
- Player height stays constant on flat ground

**Failure cases to check:**
- ❌ Player sinks into ground
- ❌ Player bounces when walking
- ❌ Player falls through specific block types

### 3. Jumping ✓

**Test:** Press jump key while on ground

**Expected behavior:**
- Player launches upward with initial velocity ~8 blocks/s
- Reaches peak height ~3.2 blocks above starting point
- Can only jump when actually on ground
- Cannot jump in mid-air (no double jump)

**Failure cases to check:**
- ❌ Can jump infinitely in air
- ❌ Jump doesn't work on certain surfaces
- ❌ Jump height is inconsistent

### 4. Ceiling Collision ✓

**Test:** Jump into a ceiling or low overhang

**Expected behavior:**
- Player stops moving upward when hitting ceiling
- Starts falling immediately after hitting ceiling
- No clipping through ceiling blocks
- Head doesn't stick in ceiling

**Failure cases to check:**
- ❌ Player clips through ceiling
- ❌ Player gets stuck in ceiling
- ❌ Player continues rising after hitting ceiling

### 5. Wall Collision ✓

**Test:** Walk into a wall

**Expected behavior:**
- Player stops at wall surface
- Cannot walk through wall
- Can slide along wall when moving at an angle
- No stuttering or vibration

**Failure cases to check:**
- ❌ Player clips through walls
- ❌ Player gets stuck in walls
- ❌ Player stutters when touching walls

### 6. Corner Sliding ✓

**Test:** Walk into a concave corner (two walls meeting)

**Expected behavior:**
- Player slides smoothly along one wall
- No getting stuck in corners
- Can eventually move away from corner
- Movement feels natural

**Failure cases to check:**
- ❌ Player gets stuck in corners
- ❌ Player clips through corner geometry
- ❌ Player stutters in corners

### 7. Block Targeting ✓

**Test:** Look at various blocks within 5 block range

**Expected behavior:**
- Targeting highlights the correct block
- Highlight disappears beyond 5 block range
- Targeting works from all angles
- Correct face is detected for each angle

**Failure cases to check:**
- ❌ Highlights wrong block
- ❌ Highlight persists beyond max range
- ❌ Targeting doesn't work at certain angles

### 8. Block Breaking ✓

**Test:** Break blocks at various positions

**Expected behavior:**
- Raycast hits the block being looked at
- Breaking works up to 5 blocks away
- Cannot break blocks beyond 5 blocks
- Breaking works from all angles

**Failure cases to check:**
- ❌ Breaks wrong block
- ❌ Can break blocks beyond 5 block range
- ❌ Cannot break certain blocks

### 9. Block Placement ✓

**Test:** Place blocks on various surfaces

**Expected behavior:**
- Block places on the face being looked at
- Block appears at correct adjacent position
- Cannot place blocks beyond 5 block range
- Cannot place blocks inside player
- Placement respects face direction

**Failure cases to check:**
- ❌ Block places inside target block
- ❌ Block places at wrong position
- ❌ Can place blocks beyond 5 block range
- ❌ Block places inside player

### 10. Stair Walking ✓

**Test:** Walk up 1-block steps

**Expected behavior:**
- Player smoothly steps up 1-block heights
- Collision detection prevents clipping
- Movement feels natural
- Can walk both up and down

**Failure cases to check:**
- ❌ Player clips through steps
- ❌ Player gets stuck on steps
- ❌ Player bounces on steps

### 11. Slope Collision ✓

**Test:** Walk against diagonal terrain

**Expected behavior:**
- Slides naturally along slopes
- Cannot walk through diagonal walls
- Smooth movement along diagonal surfaces

**Failure cases to check:**
- ❌ Clips through diagonal walls
- ❌ Gets stuck on slopes

### 12. Thin Walls ✓

**Test:** Sprint toward a 1-block-thick wall

**Expected behavior:**
- Swept collision prevents tunneling
- Cannot pass through wall at any speed
- Stops cleanly at wall surface

**Failure cases to check:**
- ❌ Clips through thin walls at high speed (tunneling)
- ❌ Passes through walls when sprinting

### 13. Water Detection ✓

**Test:** Walk into water blocks

**Expected behavior:**
- Can pass through water (water is not solid)
- Water doesn't block raycasting
- Can break/place blocks through water

**Failure cases to check:**
- ❌ Water blocks movement
- ❌ Cannot raycast through water

## Performance Verification

### Frame Rate Test

**Test:** Stand in an open area and check FPS

**Expected:** 60+ FPS with physics running

**Monitor for:**
- Consistent frame time
- No stuttering during movement
- No lag when looking at dense geometry

### Stress Test

**Test:** Create complex collision scenarios

1. Build a maze with many corners
2. Walk through rapidly
3. Monitor FPS

**Expected:**
- Minimal FPS drop
- No stuttering or freezing
- Smooth collision resolution

### Raycast Performance

**Test:** Rapidly look around in all directions

**Expected:**
- No FPS drop from raycasting
- Instant block targeting response
- No lag when scanning environment

## Edge Case Verification

### Spawn Inside Block

**Test:** Teleport player inside a solid block

**Expected behavior:**
- `resolve_penetration()` pushes player out
- Player appears at nearest air position
- No permanent stuck state

### Fall From Great Height

**Test:** Jump from build height (y=256)

**Expected behavior:**
- Falls at terminal velocity
- Lands correctly on ground
- Takes appropriate fall damage

### Zero-Width Gaps

**Test:** Try to walk through 1-block gaps while crouching isn't implemented

**Expected behavior:**
- Cannot fit through gaps smaller than player width (0.6 blocks)
- Collision detection is accurate

## Debug Visualization (Recommended)

For thorough verification, consider adding debug rendering:

### Collision AABB
- Render player's collision box as wireframe
- Verify size is 0.6 x 1.8 x 0.6
- Confirm it moves with player

### Raycast Visualization
- Render ray as a line from camera
- Show hit point with a marker
- Highlight targeted block face

### Contact Points
- Show where player is touching blocks
- Display normal vectors of collisions
- Visualize on_ground/hit_ceiling flags

## Automated Testing

For continuous verification, create automated tests:

```rust
#[test]
fn verify_cannot_clip_through_floor() {
    // Create test world with floor at y=0
    // Drop player from y=10
    // Simulate physics for 2 seconds
    // Assert player.y >= 0.9 (standing on floor)
}

#[test]
fn verify_raycast_range_limit() {
    // Create test world with wall at x=10
    // Raycast from origin toward wall
    // Assert hit is Some when distance < 5.0
    // Assert hit is None when distance > 5.0
}

#[test]
fn verify_corner_collision() {
    // Create test world with corner geometry
    // Move player into corner
    // Assert player position is outside blocks
}
```

## Issue Reporting

If verification fails, report with:

1. **What failed:** Specific test case
2. **Expected behavior:** What should happen
3. **Actual behavior:** What actually happened
4. **Reproduction steps:** Exact steps to reproduce
5. **Screenshots:** If applicable
6. **Console output:** Any error messages

## Success Criteria

Physics engine is verified when:
- ✓ All unit tests pass
- ✓ Player cannot clip through any geometry
- ✓ Gravity and jumping work correctly
- ✓ Block targeting is accurate within 5 blocks
- ✓ Block placement uses correct face detection
- ✓ Movement is smooth with no stuttering
- ✓ Performance is acceptable (60+ FPS)
- ✓ All edge cases are handled gracefully

## Verification Checklist

Use this checklist for final verification:

- [ ] Compiles without errors
- [ ] All unit tests pass
- [ ] Gravity works correctly
- [ ] Ground collision prevents floor clipping
- [ ] Jumping works only when on ground
- [ ] Ceiling collision stops upward movement
- [ ] Wall collision prevents walking through walls
- [ ] Corner sliding works smoothly
- [ ] Block targeting highlights correct block
- [ ] Raycast range limited to 5 blocks
- [ ] Block breaking hits targeted block
- [ ] Block placement uses correct face
- [ ] Cannot place blocks inside player
- [ ] Thin walls don't allow tunneling
- [ ] Water allows pass-through
- [ ] Performance is acceptable
- [ ] No stuttering or lag
- [ ] Edge cases handled (spawn in block, etc.)

Once all items are checked, the physics engine is fully verified! ✓
