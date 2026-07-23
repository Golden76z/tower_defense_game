from PIL import Image, ImageDraw

def generate_sprite():
    # 64x64 image with transparent background
    img = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Coordinates:
    # Center X is 32

    # Draw glow aura around the crystal (semi-transparent cyan circle)
    # Crystal center will be around Y = 14
    for r in range(12, 0, -2):
        alpha = int(40 * (1.0 - r / 12.0))
        draw.ellipse([32 - r, 14 - r, 32 + r, 14 + r], fill=(0, 230, 255, alpha))

    # Base (bottom block)
    # Bottom: Y=60, Top: Y=50
    # Left/Right bottom: X=12/52, Left/Right top: X=16/48
    draw.polygon([
        (12, 60), (52, 60),
        (48, 50), (16, 50)
    ], fill=(45, 50, 60, 255))
    # Base highlight (left edge)
    draw.line([(12, 60), (16, 50)], fill=(70, 75, 90, 255), width=2)
    # Base shadow (right edge)
    draw.line([(52, 60), (48, 50)], fill=(30, 32, 40, 255), width=2)

    # Shaft (middle obelisk frustum)
    # Bottom: Y=50, Top: Y=22
    # Left/Right bottom: X=18/46, Left/Right top: X=24/40
    draw.polygon([
        (18, 50), (46, 50),
        (40, 22), (24, 22)
    ], fill=(25, 20, 35, 255)) # Dark obsidian purple

    # Shaft highlight/shading:
    # Let's split it down the middle to give it a 3D faceted look
    # Left facet (highlighted):
    draw.polygon([
        (18, 50), (32, 50),
        (32, 22), (24, 22)
    ], fill=(35, 30, 50, 255))
    # Leftmost edge highlight
    draw.line([(18, 50), (24, 22)], fill=(55, 48, 75, 255), width=1)
    # Rightmost edge shadow
    draw.line([(46, 50), (40, 22)], fill=(15, 12, 22, 255), width=1)

    # Floating collar/ring
    # Y = 20 to 24, X = 20 to 44
    draw.polygon([
        (20, 24), (44, 24),
        (42, 20), (22, 20)
    ], fill=(60, 65, 80, 255))
    # Left collar highlight
    draw.polygon([
        (20, 24), (32, 24),
        (32, 20), (22, 20)
    ], fill=(80, 85, 100, 255))

    # Floating glowing Crystal (Octahedron/Diamond)
    # Y from 6 to 20
    # Left: X=26, Right: X=38, Middle Y=13
    # Top/Bottom peaks: X=32, Y=6 and Y=20
    # Upper left facet
    draw.polygon([(32, 6), (32, 13), (26, 13)], fill=(0, 230, 255, 255)) # bright cyan
    # Upper right facet
    draw.polygon([(32, 6), (38, 13), (32, 13)], fill=(0, 190, 220, 255)) # mid cyan
    # Lower left facet
    draw.polygon([(32, 20), (32, 13), (26, 13)], fill=(0, 160, 190, 255)) # dark cyan
    # Lower right facet
    draw.polygon([(32, 20), (38, 13), (32, 13)], fill=(0, 130, 160, 255)) # darker cyan

    # Center sparkle (vertical highlight line on the crystal)
    draw.line([(32, 8), (32, 18)], fill=(220, 255, 255, 255), width=1)

    # Save to file
    img.save("assets/sprites/sniper_tower.png")
    print("Successfully generated assets/sprites/sniper_tower.png")

if __name__ == '__main__':
    generate_sprite()
