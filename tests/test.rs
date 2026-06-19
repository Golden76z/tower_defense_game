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
        let texture_id = (i + 2) % 4; // texture ids: 2, 3, 0, 1
        batcher.add_cube(
            Vec3::new(0.0, i as f32, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            texture_id,
        );
    }
    let duration_add = start_add.elapsed();
    println!("Adding 1000 items took: {:?}", duration_add);

    let start_compile = Instant::now();
    batcher.compile_batches();
    let duration_compile = start_compile.elapsed();
    println!("Compiling 1000 items took: {:?}", duration_compile);

    // Verify batches
    let batches = batcher.batches();
    
    // Since texture_id can only be 0, 1, 2, or 3, sorting by texture_id
    // must result in exactly 4 batches (one for each texture_id)
    assert_eq!(batches.len(), 4);

    let mut total_vertices = 0;
    for (idx, batch) in batches.iter().enumerate() {
        assert_eq!(batch.texture_id, idx);
        assert_eq!(batch.vertex_offset, total_vertices);
        total_vertices += batch.vertex_count;
    }

    // Expected total vertices:
    // 500 sprites * 6 vertices = 3000 vertices
    // 500 cubes * 36 vertices = 18000 vertices
    // Total = 21000 vertices
    assert_eq!(total_vertices, 21000);
    assert_eq!(batcher.vertices().len(), 21000);

    // Performance assertion: Compilation on CPU should be extremely fast (under 10ms, usually <1ms)
    assert!(duration_compile.as_millis() < 10, "Compilation took too long: {:?}", duration_compile);
}

