use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

pub struct Camera {
    pub target: Vec3,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            zoom: 1.0,
        }
    }

    pub fn build_view_projection_matrix(&self, width: u32, height: u32) -> Mat4 {
        // Isometric eye offset: look at target from (target + (2.0, 2.0, 2.0))
        let eye_offset = Vec3::new(2.0, 2.0, 2.0);
        let eye = self.target + eye_offset;
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);

        let aspect = width as f32 / height as f32;
        // Base ortho height is 2.0, scale by zoom
        let ortho_height = 2.0 / self.zoom;
        let ortho_width = ortho_height * aspect;
        
        let proj = Mat4::orthographic_rh(
            -ortho_width / 2.0,
            ortho_width / 2.0,
            -ortho_height / 2.0,
            ortho_height / 2.0,
            -10.0,
            10.0,
        );

        proj * view
    }

    /// Builds the world-space picking ray for a cursor position (physical
    /// pixels, origin top-left).
    ///
    /// Returns `(origin, direction)` where `origin` is the point on the near
    /// plane and `direction` is normalised. `None` if the math degenerates.
    pub fn screen_ray(
        &self,
        screen_x: f32,
        screen_y: f32,
        width: u32,
        height: u32,
    ) -> Option<(Vec3, Vec3)> {
        if width == 0 || height == 0 {
            return None;
        }

        let inv = self.build_view_projection_matrix(width, height).inverse();

        // Pixel -> normalised device coordinates. Screen Y points down, NDC Y
        // points up, so it's flipped.
        let ndc_x = 2.0 * screen_x / width as f32 - 1.0;
        let ndc_y = 1.0 - 2.0 * screen_y / height as f32;

        // wgpu clip space has z in [0, 1]: z=0 is the near plane, z=1 the far.
        let near = inv * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let far = inv * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        if near.w.abs() < 1e-8 || far.w.abs() < 1e-8 {
            return None;
        }

        let near = near.xyz() / near.w;
        let far = far.xyz() / far.w;
        let dir = (far - near).normalize_or_zero();
        if dir == Vec3::ZERO {
            return None;
        }

        Some((near, dir))
    }

    /// Unprojects a cursor position into the world point where its ray crosses
    /// the horizontal plane `y = plane_y`. Useful for ground decals.
    ///
    /// Returns `None` if the ray is parallel to the plane (which shouldn't
    /// happen for the isometric camera) or the math degenerates.
    pub fn screen_to_ground(
        &self,
        screen_x: f32,
        screen_y: f32,
        width: u32,
        height: u32,
        plane_y: f32,
    ) -> Option<Vec3> {
        let (origin, dir) = self.screen_ray(screen_x, screen_y, width, height)?;
        if dir.y.abs() < 1e-8 {
            return None; // Ray parallel to the ground plane.
        }
        let t = (plane_y - origin.y) / dir.y;
        Some(origin + dir * t)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CameraController {
    pub move_up: bool,
    pub move_down: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub zoom_delta: f32,
    pub speed: f32,
    pub zoom_speed: f32,
}

impl CameraController {
    pub fn new(speed: f32, zoom_speed: f32) -> Self {
        Self {
            move_up: false,
            move_down: false,
            move_left: false,
            move_right: false,
            zoom_in: false,
            zoom_out: false,
            zoom_delta: 0.0,
            speed,
            zoom_speed,
        }
    }

    pub fn process_key(&mut self, key_code: winit::keyboard::KeyCode, is_pressed: bool) -> bool {
        use winit::keyboard::KeyCode;
        match key_code {
            KeyCode::KeyW | KeyCode::KeyZ => {
                self.move_up = is_pressed;
                true
            }
            KeyCode::KeyS => {
                self.move_down = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::KeyQ => {
                self.move_left = is_pressed;
                true
            }
            KeyCode::KeyD => {
                self.move_right = is_pressed;
                true
            }
            KeyCode::Equal | KeyCode::NumpadAdd => {
                self.zoom_in = is_pressed;
                true
            }
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                self.zoom_out = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_scroll(&mut self, delta: &winit::event::MouseScrollDelta) {
        use winit::event::MouseScrollDelta;
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.zoom_delta += y * 0.1;
            }
            MouseScrollDelta::PixelDelta(pos) => {
                self.zoom_delta += pos.y as f32 * 0.005;
            }
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        // Compute movement direction aligned to X-Z ground plane under isometric view.
        // Right vector: positive X, negative Z
        let right_vec = Vec3::new(1.0, 0.0, -1.0).normalize();
        // Up vector: negative X, negative Z (towards the upper part of the screen)
        let up_vec = Vec3::new(-1.0, 0.0, -1.0).normalize();

        let mut move_dir = Vec3::ZERO;
        if self.move_up {
            move_dir += up_vec;
        }
        if self.move_down {
            move_dir -= up_vec;
        }
        if self.move_left {
            move_dir -= right_vec;
        }
        if self.move_right {
            move_dir += right_vec;
        }

        // Normalize direction if moving to prevent diagonal speedup
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // Speed is scaled by zoom so movement feels consistent at any scale
        let current_speed = (self.speed / camera.zoom.max(0.1)) * dt;
        camera.target += move_dir * current_speed;

        // Process zoom inputs
        let mut zoom_change = 0.0;
        if self.zoom_in {
            zoom_change += self.zoom_speed * dt;
        }
        if self.zoom_out {
            zoom_change -= self.zoom_speed * dt;
        }
        zoom_change += self.zoom_delta;
        self.zoom_delta = 0.0; // Clear scroll delta for the frame

        if zoom_change != 0.0 {
            // Logarithmic/proportional zoom so zoom speed stays proportional to zoom depth
            camera.zoom += camera.zoom * zoom_change;
            camera.zoom = camera.zoom.clamp(0.1, 10.0);
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new(2.0, 1.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4Swizzles;

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_zoom_projections() {
        let camera = Camera {
            target: Vec3::ZERO,
            zoom: 2.0,
        };
        let mat = camera.build_view_projection_matrix(800, 600);
        // Ensure projection is not NAN or INF
        assert!(mat.is_finite());
    }

    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::new();
        let mut controller = CameraController::new(10.0, 1.0);
        controller.move_right = true;
        controller.update_camera(&mut camera, 0.1);
        
        // Moving right should increase x and decrease z in world space
        assert!(camera.target.x > 0.0);
        assert!(camera.target.z < 0.0);
        assert_eq!(camera.target.y, 0.0);
    }

    #[test]
    fn test_screen_center_maps_to_target_on_ground() {
        // The screen centre should unproject onto the camera target (which sits
        // on the ground plane y = 0).
        let camera = Camera::new();
        let (w, h) = (800u32, 600u32);
        let hit = camera
            .screen_to_ground(w as f32 / 2.0, h as f32 / 2.0, w, h, 0.0)
            .expect("centre ray should hit the ground");
        assert!(hit.x.abs() < 1e-3, "x ~ 0, got {}", hit.x);
        assert!(hit.z.abs() < 1e-3, "z ~ 0, got {}", hit.z);
        assert!(hit.y.abs() < 1e-3, "y ~ 0, got {}", hit.y);
    }

    #[test]
    fn test_screen_to_ground_follows_camera_target() {
        // When the camera target moves, the centre ray should track it.
        let camera = Camera {
            target: Vec3::new(1.5, 0.0, -2.0),
            zoom: 1.0,
        };
        let (w, h) = (800u32, 600u32);
        let hit = camera
            .screen_to_ground(w as f32 / 2.0, h as f32 / 2.0, w, h, 0.0)
            .unwrap();
        assert!((hit.x - 1.5).abs() < 1e-3, "x, got {}", hit.x);
        assert!((hit.z + 2.0).abs() < 1e-3, "z, got {}", hit.z);
    }

    #[test]
    fn test_sprite_centering_when_camera_moves() {
        let mut camera = Camera::new();
        
        // 1. Initially target is ZERO. Projecting target (ZERO) should be at center (0,0) in NDC space.
        let mat = camera.build_view_projection_matrix(800, 600);
        let clip_pos = mat * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        let ndc = clip_pos.xyz() / clip_pos.w;
        assert!((ndc.x - 0.0).abs() < 1e-5);
        assert!((ndc.y - 0.0).abs() < 1e-5);

        // 2. Now move camera target to some arbitrary position.
        camera.target = Vec3::new(12.3, 4.5, -6.7);
        let mat = camera.build_view_projection_matrix(800, 600);
        
        // Projecting the sprite at (12.3, 4.5, -6.7) should still map to NDC center (0,0).
        let clip_pos = mat * glam::Vec4::new(12.3, 4.5, -6.7, 1.0);
        let ndc = clip_pos.xyz() / clip_pos.w;
        assert!((ndc.x - 0.0).abs() < 1e-5);
        assert!((ndc.y - 0.0).abs() < 1e-5);
    }
}

