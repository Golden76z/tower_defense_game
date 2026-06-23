use glam::{Mat4, Vec3, Vec4};

#[test]
fn test_vertex_projection() {
    let width = 800;
    let height = 600;
    
    // Camera view-proj
    let eye = Vec3::new(2.0, 2.0, 2.0);
    let target = Vec3::ZERO;
    let up = Vec3::Y;
    let view = Mat4::look_at_rh(eye, target, up);

    let aspect = width as f32 / height as f32;
    let ortho_height = 2.0;
    let ortho_width = ortho_height * aspect;
    let proj = Mat4::orthographic_rh(
        -ortho_width / 2.0,
        ortho_width / 2.0,
        -ortho_height / 2.0,
        ortho_height / 2.0,
        -10.0,
        10.0,
    );

    let view_proj = proj * view;

    let vertices = [
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(-0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, -0.5),
    ];

    println!("--- Transformed coordinates (Clip Space) ---");
    for (i, &v) in vertices.iter().enumerate() {
        let clip_pos = view_proj * Vec4::from((v, 1.0));
        println!(
            "Vertex {}: World {:?} -> Clip [{:.4}, {:.4}, {:.4}, {:.4}] (NDC: [{:.4}, {:.4}, {:.4}])",
            i,
            v,
            clip_pos.x,
            clip_pos.y,
            clip_pos.z,
            clip_pos.w,
            clip_pos.x / clip_pos.w,
            clip_pos.y / clip_pos.w,
            clip_pos.z / clip_pos.w
        );
    }
}

#[test]
fn test_batcher_performance_and_correctness() {
    use tower_defense_lib::renderer::batch::SpriteBatcher;
    use tower_defense_lib::renderer::sprite::{Sprite, SpriteAlignment};
    use glam::{Vec2, Vec3};
    use std::time::Instant;

    let mut batcher = SpriteBatcher::new();
    
    // Add 500 sprites and 500 cubes (total 1000 items)
    // Alternate texture IDs 0, 1, 2, 3 to test sorting
    let num_sprites = 500;
    let num_cubes = 500;

    let start_add = Instant::now();
    for i in 0..num_sprites {
        let texture_id = i % 4;
        let sprite = Sprite::new(
            Vec3::new(i as f32, 0.0, 0.0),
            Vec2::new(1.0, 1.0),
            texture_id,
            0.0,
        );
        batcher.add_sprite(sprite, SpriteAlignment::Horizontal);
    }

    for i in 0..num_cubes {
        let side_tex = (i + 2) % 4;
        let top_tex = (i + 3) % 4;
        batcher.add_cube(
            Vec3::new(0.0, i as f32, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            side_tex,
            top_tex,
        );
    }
    let duration_add = start_add.elapsed();
    println!("Adding 1000 items took: {:?}", duration_add);

    let start_compile = Instant::now();
    batcher.compile_batches();
    let duration_compile = start_compile.elapsed();
    println!("Compiling 1500 items took: {:?}", duration_compile);

    // Verify batches
    let batches = batcher.batches();
    
    // Texture IDs 0-3 used by sprites and cube sides/tops
    // All 4 texture IDs should be present
    assert_eq!(batches.len(), 4);

    let mut total_vertices = 0;
    for (idx, batch) in batches.iter().enumerate() {
        assert_eq!(batch.texture_id, idx);
        assert_eq!(batch.vertex_offset, total_vertices);
        total_vertices += batch.vertex_count;
    }

    // Expected total vertices:
    // 500 sprites * 6 vertices = 3000 vertices
    // 500 cubes: sides = 500 * 30 = 15000, tops = 500 * 6 = 3000
    // Total = 3000 + 15000 + 3000 = 21000 vertices
    assert_eq!(total_vertices, 21000);
    assert_eq!(batcher.vertices().len(), 21000);

    // Performance assertion: Compilation on CPU should be extremely fast (under 10ms, usually <1ms)
    assert!(duration_compile.as_millis() < 10, "Compilation took too long: {:?}", duration_compile);
}

#[test]
fn test_health_bar_color_interpolation() {
    let interpolate = |health_pct: f32| -> [f32; 4] {
        let r = if health_pct > 0.5 {
            2.0 * (1.0 - health_pct)
        } else {
            1.0
        };
        let g = if health_pct > 0.5 {
            1.0
        } else {
            2.0 * health_pct
        };
        let b = 0.0;
        [r, g, b, 1.0]
    };

    // 100% health should be Green [0.0, 1.0, 0.0, 1.0]
    assert_eq!(interpolate(1.0), [0.0, 1.0, 0.0, 1.0]);

    // 50% health should be Yellow [1.0, 1.0, 0.0, 1.0]
    assert_eq!(interpolate(0.5), [1.0, 1.0, 0.0, 1.0]);

    // 0% health should be Red [1.0, 0.0, 0.0, 1.0]
    assert_eq!(interpolate(0.0), [1.0, 0.0, 0.0, 1.0]);

    // 75% health should be intermediate orange-green/yellowish
    let col_75 = interpolate(0.75);
    assert!(col_75[0] > 0.0 && col_75[0] < 1.0);
    assert_eq!(col_75[1], 1.0);

    // 25% health should be intermediate red-orange
    let col_25 = interpolate(0.25);
    assert_eq!(col_25[0], 1.0);
    assert!(col_25[1] > 0.0 && col_25[1] < 1.0);
}

