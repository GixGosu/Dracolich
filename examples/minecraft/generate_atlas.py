#!/usr/bin/env python3
"""
Procedural texture atlas generator
Creates a 256x256 PNG texture atlas with 16x16 textures for all block types
Run this to generate assets/atlas.png
"""

from PIL import Image, ImageDraw
import random
import os

ATLAS_SIZE = 256
TILE_SIZE = 16
TILES_PER_ROW = ATLAS_SIZE // TILE_SIZE  # 16

# Color constants (R, G, B, A)
BLACK = (0, 0, 0, 255)
GRASS_GREEN = (92, 160, 55, 255)
DIRT_BROWN = (134, 96, 67, 255)
STONE_GRAY = (128, 128, 128, 255)
COBBLE_GRAY = (110, 110, 110, 255)
SAND_TAN = (218, 210, 158, 255)
GRAVEL_GRAY = (136, 126, 126, 255)
BEDROCK_DARK = (64, 64, 64, 255)
BARK_BROWN = (102, 76, 51, 255)
WOOD_LIGHT = (157, 128, 79, 255)
BIRCH_WHITE = (216, 215, 201, 255)
BIRCH_MARK = (60, 60, 60, 255)
LEAF_GREEN = (76, 150, 24, 255)
LEAF_BIRCH = (128, 167, 85, 255)
WATER_BLUE = (32, 64, 255, 180)
GLASS_CYAN = (200, 230, 255, 120)
COAL_BLACK = (45, 45, 45, 255)
IRON_TAN = (196, 181, 171, 255)
GOLD_YELLOW = (255, 215, 0, 255)
DIAMOND_CYAN = (93, 219, 213, 255)
PLANK_BROWN = (162, 130, 78, 255)
FURNACE_GRAY = (96, 96, 96, 255)
FURNACE_DARK = (32, 32, 32, 255)


def noise_color(base, variance, rng):
    """Add noise to a color"""
    r = max(0, min(255, base[0] + rng.randint(-variance, variance)))
    g = max(0, min(255, base[1] + rng.randint(-variance, variance)))
    b = max(0, min(255, base[2] + rng.randint(-variance, variance)))
    return (r, g, b, base[3])


def fill_tile(color):
    """Create a tile filled with a solid color"""
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE), color)
    return tile


def tile_air():
    return fill_tile(BLACK)


def tile_grass_top(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 3 == 0:
                tile.putpixel((x, y), noise_color(GRASS_GREEN, 15, rng))
            else:
                tile.putpixel((x, y), noise_color(GRASS_GREEN, 8, rng))
    return tile


def tile_dirt(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            tile.putpixel((x, y), noise_color(DIRT_BROWN, 12, rng))
    return tile


def tile_grass_side(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if y < 2:
                tile.putpixel((x, y), noise_color(GRASS_GREEN, 10, rng))
            else:
                tile.putpixel((x, y), noise_color(DIRT_BROWN, 10, rng))
    return tile


def tile_stone(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            base = noise_color(STONE_GRAY, 15, rng)
            # Add darker speckles
            if rng.random() < 0.16:
                base = (max(0, base[0] - 30), max(0, base[1] - 30), max(0, base[2] - 30), base[3])
            tile.putpixel((x, y), base)
    return tile


def tile_cobblestone(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            edge = (x % 4 == 0 or y % 4 == 0) and rng.random() < 0.66
            if edge:
                tile.putpixel((x, y), (60, 60, 60, 255))
            else:
                tile.putpixel((x, y), noise_color(COBBLE_GRAY, 20, rng))
    return tile


def tile_sand(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            tile.putpixel((x, y), noise_color(SAND_TAN, 10, rng))
    return tile


def tile_gravel(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 2 == 0:
                tile.putpixel((x, y), noise_color(GRAVEL_GRAY, 20, rng))
            else:
                tile.putpixel((x, y), noise_color(GRAVEL_GRAY, 15, rng))
    return tile


def tile_bedrock(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            pattern = ((x // 2 + y // 2) % 3) - 1
            offset = pattern * 15
            r = max(0, min(255, BEDROCK_DARK[0] + offset))
            g = max(0, min(255, BEDROCK_DARK[1] + offset))
            b = max(0, min(255, BEDROCK_DARK[2] + offset))
            tile.putpixel((x, y), (r, g, b, 255))
    return tile


def tile_oak_log_top(rng):
    tile = fill_tile(WOOD_LIGHT)
    draw = ImageDraw.Draw(tile)
    # Draw concentric circles (tree rings)
    for r in range(1, 6):
        draw.ellipse([8-r, 8-r, 8+r, 8+r], outline=noise_color(BARK_BROWN, 10, rng))
    return tile


def tile_oak_log_side(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if y % 3 == 0:
                tile.putpixel((x, y), noise_color(BARK_BROWN, 15, rng))
            else:
                tile.putpixel((x, y), noise_color(BARK_BROWN, 20, rng))
    return tile


def tile_birch_log_top(rng):
    tile = fill_tile(WOOD_LIGHT)
    draw = ImageDraw.Draw(tile)
    for r in range(1, 6):
        draw.ellipse([8-r, 8-r, 8+r, 8+r], outline=noise_color(BIRCH_WHITE, 8, rng))
    return tile


def tile_birch_log_side(rng):
    tile = fill_tile(BIRCH_WHITE)
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if rng.random() < 0.125:
                tile.putpixel((x, y), BIRCH_MARK)
    return tile


def tile_leaves_oak(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 2 == 0 and rng.random() < 0.33:
                tile.putpixel((x, y), (0, 0, 0, 0))  # Transparent
            else:
                tile.putpixel((x, y), noise_color(LEAF_GREEN, 20, rng))
    return tile


def tile_leaves_birch(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 2 == 0 and rng.random() < 0.33:
                tile.putpixel((x, y), (0, 0, 0, 0))
            else:
                tile.putpixel((x, y), noise_color(LEAF_BIRCH, 20, rng))
    return tile


def tile_water(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 4 == 0:
                tile.putpixel((x, y), (40, 80, 255, 180))
            else:
                tile.putpixel((x, y), noise_color(WATER_BLUE, 10, rng))
    return tile


def tile_glass(rng):
    return fill_tile(GLASS_CYAN)


def tile_ore(base_stone, ore_color, rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            is_ore = ((x // 2 + y // 3) % 5 == 0) or ((x // 3 + y // 2) % 7 == 1)
            if is_ore:
                tile.putpixel((x, y), noise_color(ore_color, 15, rng))
            else:
                tile.putpixel((x, y), noise_color(base_stone, 12, rng))
    return tile


def tile_ore_coal(rng):
    return tile_ore(STONE_GRAY, COAL_BLACK, rng)


def tile_ore_iron(rng):
    return tile_ore(STONE_GRAY, IRON_TAN, rng)


def tile_ore_gold(rng):
    return tile_ore(STONE_GRAY, GOLD_YELLOW, rng)


def tile_ore_diamond(rng):
    return tile_ore(STONE_GRAY, DIAMOND_CYAN, rng)


def tile_planks(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if y % 4 == 0:
                darker = (PLANK_BROWN[0] - 20, PLANK_BROWN[1] - 20, PLANK_BROWN[2] - 20, 255)
                tile.putpixel((x, y), darker)
            else:
                tile.putpixel((x, y), noise_color(PLANK_BROWN, 10, rng))
    return tile


def tile_crafting_table_top(rng):
    tile = tile_planks(rng)
    draw = ImageDraw.Draw(tile)
    # Draw 2x2 grid
    draw.line([(7, 0), (7, TILE_SIZE-1)], fill=(80, 60, 40, 255), width=2)
    draw.line([(0, 7), (TILE_SIZE-1, 7)], fill=(80, 60, 40, 255), width=2)
    return tile


def tile_crafting_table_side(rng):
    tile = tile_planks(rng)
    draw = ImageDraw.Draw(tile)
    # Draw simple tools
    draw.rectangle([2, 4, 5, 6], fill=(160, 160, 160, 255))  # Tool head
    draw.line([(4, 7), (4, 11)], fill=(120, 90, 60, 255), width=1)  # Handle
    return tile


def tile_furnace_front(rng):
    tile = fill_tile(FURNACE_GRAY)
    draw = ImageDraw.Draw(tile)
    # Draw opening
    draw.rectangle([5, 5, 10, 10], fill=FURNACE_DARK)
    return tile


def tile_furnace_top(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            tile.putpixel((x, y), noise_color(FURNACE_GRAY, 10, rng))
    return tile


def tile_furnace_side(rng):
    tile = Image.new('RGBA', (TILE_SIZE, TILE_SIZE))
    for y in range(TILE_SIZE):
        for x in range(TILE_SIZE):
            if (x + y) % 4 == 0:
                tile.putpixel((x, y), noise_color(FURNACE_GRAY, 15, rng))
            else:
                tile.putpixel((x, y), noise_color(FURNACE_GRAY, 8, rng))
    return tile


def generate_tile(atlas, index, tile):
    """Place a 16x16 tile at the given index in the atlas"""
    tile_x = (index % TILES_PER_ROW) * TILE_SIZE
    tile_y = (index // TILES_PER_ROW) * TILE_SIZE
    atlas.paste(tile, (tile_x, tile_y))


def main():
    print("Generating texture atlas...")

    # Create atlas
    atlas = Image.new('RGBA', (ATLAS_SIZE, ATLAS_SIZE), (0, 0, 0, 255))

    # Use deterministic random for reproducibility
    rng = random.Random(42)

    # Generate all tiles
    generate_tile(atlas, 0, tile_air())
    generate_tile(atlas, 1, tile_grass_top(rng))
    generate_tile(atlas, 2, tile_dirt(rng))
    generate_tile(atlas, 3, tile_grass_side(rng))
    generate_tile(atlas, 4, tile_stone(rng))
    generate_tile(atlas, 5, tile_cobblestone(rng))
    generate_tile(atlas, 6, tile_sand(rng))
    generate_tile(atlas, 7, tile_gravel(rng))
    generate_tile(atlas, 8, tile_bedrock(rng))
    generate_tile(atlas, 9, tile_oak_log_top(rng))
    generate_tile(atlas, 10, tile_oak_log_side(rng))
    generate_tile(atlas, 11, tile_birch_log_top(rng))
    generate_tile(atlas, 12, tile_birch_log_side(rng))
    generate_tile(atlas, 13, tile_leaves_oak(rng))
    generate_tile(atlas, 14, tile_leaves_birch(rng))
    generate_tile(atlas, 15, tile_water(rng))
    generate_tile(atlas, 16, tile_glass(rng))
    generate_tile(atlas, 17, tile_ore_coal(rng))
    generate_tile(atlas, 18, tile_ore_iron(rng))
    generate_tile(atlas, 19, tile_ore_gold(rng))
    generate_tile(atlas, 20, tile_ore_diamond(rng))
    generate_tile(atlas, 21, tile_planks(rng))
    generate_tile(atlas, 22, tile_crafting_table_top(rng))
    generate_tile(atlas, 24, tile_crafting_table_side(rng))
    generate_tile(atlas, 25, tile_furnace_front(rng))
    generate_tile(atlas, 26, tile_furnace_top(rng))
    generate_tile(atlas, 27, tile_furnace_side(rng))

    # Save to file
    os.makedirs('assets', exist_ok=True)
    output_path = 'assets/atlas.png'
    atlas.save(output_path)
    print(f"Atlas saved to {output_path}")
    print(f"Atlas size: {ATLAS_SIZE}x{ATLAS_SIZE}")
    print(f"Tile size: {TILE_SIZE}x{TILE_SIZE}")
    print(f"Tiles generated: 28")


if __name__ == '__main__':
    main()
