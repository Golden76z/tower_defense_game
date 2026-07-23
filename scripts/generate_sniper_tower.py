import os

def generate_sniper_tower():
    os.makedirs('assets/models', exist_ok=True)
    
    vertices = []
    faces = []
    
    def add_quad(v1, v2, v3, v4, r, g, b):
        # We add 4 vertices and a quad face.
        # To get flat shading, we duplicate the vertices so they can have the face's color and normal.
        idx = len(vertices) + 1
        vertices.extend([
            f"v {v1[0]:.4f} {v1[1]:.4f} {v1[2]:.4f} {r:.4f} {g:.4f} {b:.4f}",
            f"v {v2[0]:.4f} {v2[1]:.4f} {v2[2]:.4f} {r:.4f} {g:.4f} {b:.4f}",
            f"v {v3[0]:.4f} {v3[1]:.4f} {v3[2]:.4f} {r:.4f} {g:.4f} {b:.4f}",
            f"v {v4[0]:.4f} {v4[1]:.4f} {v4[2]:.4f} {r:.4f} {g:.4f} {b:.4f}"
        ])
        faces.append(f"f {idx} {idx+1} {idx+2} {idx+3}")

    def add_tri(v1, v2, v3, r, g, b):
        idx = len(vertices) + 1
        vertices.extend([
            f"v {v1[0]:.4f} {v1[1]:.4f} {v1[2]:.4f} {r:.4f} {g:.4f} {b:.4f}",
            f"v {v2[0]:.4f} {v2[1]:.4f} {v2[2]:.4f} {r:.4f} {g:.4f} {b:.4f}",
            f"v {v3[0]:.4f} {v3[1]:.4f} {v3[2]:.4f} {r:.4f} {g:.4f} {b:.4f}"
        ])
        faces.append(f"f {idx} {idx+1} {idx+2}")

    # Colors (R, G, B)
    # Dark base stone
    c_base_top = (0.25, 0.28, 0.32)
    c_base_side1 = (0.20, 0.22, 0.25)
    c_base_side2 = (0.16, 0.18, 0.20)
    c_base_side3 = (0.22, 0.24, 0.27)
    
    # Obsidian shaft (deep purple-black)
    c_shaft1 = (0.12, 0.10, 0.16)
    c_shaft2 = (0.08, 0.06, 0.12)
    c_shaft3 = (0.15, 0.12, 0.20)
    c_shaft4 = (0.10, 0.08, 0.14)

    # Cyan/Neon accents/gem (glowing sniper focus crystal)
    c_gem = (0.0, 0.85, 0.95)
    c_gem_bright = (0.3, 0.95, 1.0)
    c_gem_dark = (0.0, 0.65, 0.75)

    # 1. Base (bottom block)
    # Bottom: Y=0.0, Top: Y=0.25
    b_bot = [
        (-0.45, 0.0, -0.45), # 0
        (0.45, 0.0, -0.45),  # 1
        (0.45, 0.0, 0.45),   # 2
        (-0.45, 0.0, 0.45)   # 3
    ]
    b_top = [
        (-0.4, 0.25, -0.4),  # 4
        (0.4, 0.25, -0.4),   # 5
        (0.4, 0.25, 0.4),    # 6
        (-0.4, 0.25, 0.4)    # 7
    ]
    # Base Bottom face (facing down)
    add_quad(b_bot[3], b_bot[2], b_bot[1], b_bot[0], *c_base_side2)
    # Base Top face
    add_quad(b_top[0], b_top[1], b_top[2], b_top[3], *c_base_top)
    # Base Sides
    # Front: bot0, bot1, top1, top0 -> index 0, 1, 5, 4
    add_quad(b_bot[0], b_bot[1], b_top[1], b_top[0], *c_base_side1)
    # Right: bot1, bot2, top2, top1 -> index 1, 2, 6, 5
    add_quad(b_bot[1], b_bot[2], b_top[2], b_top[1], *c_base_side2)
    # Back: bot2, bot3, top3, top2 -> index 2, 3, 7, 6
    add_quad(b_bot[2], b_bot[3], b_top[3], b_top[2], *c_base_side3)
    # Left: bot3, bot0, top0, top3 -> index 3, 0, 4, 7
    add_quad(b_bot[3], b_bot[0], b_top[0], b_top[3], *c_base_side2)

    # 2. Shaft (middle tall obelisk frustum)
    # Bottom: Y=0.25, Top: Y=1.1
    s_bot = [
        (-0.3, 0.25, -0.3),  # 0
        (0.3, 0.25, -0.3),   # 1
        (0.3, 0.25, 0.3),    # 2
        (-0.3, 0.25, 0.3)    # 3
    ]
    s_top = [
        (-0.15, 1.1, -0.15), # 4
        (0.15, 1.1, -0.15),  # 5
        (0.15, 1.1, 0.15),   # 6
        (-0.15, 1.1, 0.15)   # 7
    ]
    # Shaft Sides (Obelisk tapering)
    add_quad(s_bot[0], s_bot[1], s_top[1], s_top[0], *c_shaft1)
    add_quad(s_bot[1], s_bot[2], s_top[2], s_top[1], *c_shaft2)
    add_quad(s_bot[2], s_bot[3], s_top[3], s_top[2], *c_shaft3)
    add_quad(s_bot[3], s_bot[0], s_top[0], s_top[3], *c_shaft4)

    # 3. Floating Ring (energy band around the floating crystal)
    # Bottom: Y=1.18, Top: Y=1.28
    r_bot = [
        (-0.22, 1.18, -0.22),
        (0.22, 1.18, -0.22),
        (0.22, 1.18, 0.22),
        (-0.22, 1.18, 0.22)
    ]
    r_top = [
        (-0.22, 1.28, -0.22),
        (0.22, 1.28, -0.22),
        (0.22, 1.28, 0.22),
        (-0.22, 1.28, 0.22)
    ]
    # Ring outer sides (to look like a metallic collar)
    add_quad(r_bot[0], r_bot[1], r_top[1], r_top[0], *c_base_top)
    add_quad(r_bot[1], r_bot[2], r_top[2], r_top[1], *c_base_side1)
    add_quad(r_bot[2], r_bot[3], r_top[3], r_top[2], *c_base_side2)
    add_quad(r_bot[3], r_bot[0], r_top[0], r_top[3], *c_base_side3)

    # 4. Floating glowing Crystal (Octahedron / Diamond shape floating at the center of the ring)
    # Center is at Y=1.28
    # Bottom point: Y=1.12
    # Middle ring: Y=1.28 (X/Z +/- 0.12)
    # Top point: Y=1.44
    c_bot = (0.0, 1.12, 0.0)
    c_mid = [
        (-0.12, 1.28, -0.12), # 0
        (0.12, 1.28, -0.12),  # 1
        (0.12, 1.28, 0.12),   # 2
        (-0.12, 1.28, 0.12)   # 3
    ]
    c_top = (0.0, 1.44, 0.0)

    # Lower diamond faces (facing downwards towards s_top)
    add_tri(c_bot, c_mid[1], c_mid[0], *c_gem_dark)
    add_tri(c_bot, c_mid[2], c_mid[1], *c_gem)
    add_tri(c_bot, c_mid[3], c_mid[2], *c_gem_dark)
    add_tri(c_bot, c_mid[0], c_mid[3], *c_gem)

    # Upper diamond faces (facing upwards)
    add_tri(c_top, c_mid[0], c_mid[1], *c_gem_bright)
    add_tri(c_top, c_mid[1], c_mid[2], *c_gem)
    add_tri(c_top, c_mid[2], c_mid[3], *c_gem_bright)
    add_tri(c_top, c_mid[3], c_mid[0], *c_gem)

    # Write OBJ
    with open('assets/models/sniper_tower.obj', 'w') as f:
        f.write("# Sniper Tower Model with baked vertex colors\n")
        for v in vertices:
            f.write(v + "\n")
        f.write("\n")
        for face in faces:
            f.write(face + "\n")

    print("Successfully generated assets/models/sniper_tower.obj")

if __name__ == '__main__':
    generate_sniper_tower()
