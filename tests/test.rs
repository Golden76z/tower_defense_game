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
