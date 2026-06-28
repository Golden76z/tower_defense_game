use glam::{Vec2, Vec3};
use crate::renderer::sprite::{Sprite, SpriteAlignment};

/// A Linear Congruential Generator (LCG) for fast, allocation-free pseudo-random values.
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0x7FFFFFFF) as f32 / 2147483647.0
    }

    pub fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub size_start: Vec2,
    pub size_end: Vec2,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub alignment: SpriteAlignment,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    rng: SimpleRng,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(1024),
            rng: SimpleRng::new(1337),
        }
    }

    /// Updates all active particles, moving them, updating their lifetime, and removing dead ones.
    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.rotation += p.rotation_speed * dt;
            p.lifetime -= dt;
        }

        // Keep only particles that are still alive
        self.particles.retain(|p| p.lifetime > 0.0);
    }

    /// Queues all active particles into the sprite batcher.
    pub fn draw(
        &self,
        batcher: &mut crate::renderer::batch::SpriteBatcher,
        texture_id: usize,
        cull_params: Option<(glam::Mat4, f32, f32)>,
    ) {
        for p in &self.particles {
            if let Some((vp, ortho_width, ortho_height)) = cull_params {
                let size_max = p.size_start.max_element();
                if !crate::renderer::camera::Camera::is_visible_with_vp(
                    vp,
                    p.position,
                    size_max * 1.5,
                    ortho_width,
                    ortho_height,
                ) {
                    continue;
                }
            }

            let progress = (1.0 - (p.lifetime / p.max_lifetime)).clamp(0.0, 1.0);
            
            // Lerp size
            let current_size = Vec2::new(
                p.size_start.x + (p.size_end.x - p.size_start.x) * progress,
                p.size_start.y + (p.size_end.y - p.size_start.y) * progress,
            );

            // Lerp color
            let current_color = [
                p.color_start[0] + (p.color_end[0] - p.color_start[0]) * progress,
                p.color_start[1] + (p.color_end[1] - p.color_start[1]) * progress,
                p.color_start[2] + (p.color_end[2] - p.color_start[2]) * progress,
                p.color_start[3] + (p.color_end[3] - p.color_start[3]) * progress,
            ];

            let sprite = Sprite::new_colored(
                p.position,
                current_size,
                texture_id,
                p.rotation,
                current_color,
            );

            batcher.add_sprite(sprite, p.alignment);
        }
    }

    /// Spawns a burst of muzzle flash sparks and smoke when a tower shoots.
    pub fn spawn_muzzle_flash(&mut self, pos: Vec3, direction: Vec2) {
        // Compute 3D direction vector in X-Z plane
        let dir_3d = Vec3::new(direction.x, 0.0, direction.y).normalize();

        // 1. Sparks (Orange/Yellow sparks moving in the firing direction)
        let spark_count = self.rng.next_f32_range(6.0, 10.0) as usize;
        for _ in 0..spark_count {
            // Cone variation
            let angle_offset = self.rng.next_f32_range(-0.3, 0.3);
            let speed = self.rng.next_f32_range(0.4, 1.2);

            // Rotate dir_3d around Y axis by angle_offset
            let cos_a = angle_offset.cos();
            let sin_a = angle_offset.sin();
            let vel_dir = Vec3::new(
                dir_3d.x * cos_a - dir_3d.z * sin_a,
                self.rng.next_f32_range(-0.05, 0.15), // some vertical scatter
                dir_3d.x * sin_a + dir_3d.z * cos_a,
            ).normalize();

            let max_lifetime = self.rng.next_f32_range(0.12, 0.22);
            let size = self.rng.next_f32_range(0.008, 0.015);

            self.particles.push(Particle {
                position: pos + vel_dir * 0.01,
                velocity: vel_dir * speed,
                color_start: [1.0, 0.9, 0.3, 1.0], // Yellowish-white
                color_end: [1.0, 0.3, 0.0, 0.0],   // Fades to transparent red
                size_start: Vec2::new(size, size),
                size_end: Vec2::new(size * 0.2, size * 0.2),
                rotation: self.rng.next_f32_range(0.0, 6.28),
                rotation_speed: self.rng.next_f32_range(-4.0, 4.0),
                lifetime: max_lifetime,
                max_lifetime,
                alignment: SpriteAlignment::Billboard,
            });
        }

        // 2. Smoke puff (small expansion, drifts up and fades out)
        let smoke_count = self.rng.next_f32_range(2.0, 4.0) as usize;
        for _ in 0..smoke_count {
            let speed = self.rng.next_f32_range(0.08, 0.2);
            let vel = Vec3::new(
                dir_3d.x * speed + self.rng.next_f32_range(-0.02, 0.02),
                self.rng.next_f32_range(0.1, 0.2), // drifts upwards
                dir_3d.z * speed + self.rng.next_f32_range(-0.02, 0.02),
            );

            let max_lifetime = self.rng.next_f32_range(0.3, 0.5);
            let start_size = self.rng.next_f32_range(0.012, 0.018);

            self.particles.push(Particle {
                position: pos,
                velocity: vel,
                color_start: [0.7, 0.7, 0.7, 0.6],
                color_end: [0.3, 0.3, 0.3, 0.0],
                size_start: Vec2::new(start_size, start_size),
                size_end: Vec2::new(start_size * 2.0, start_size * 2.0),
                rotation: self.rng.next_f32_range(0.0, 6.28),
                rotation_speed: self.rng.next_f32_range(-1.0, 1.0),
                lifetime: max_lifetime,
                max_lifetime,
                alignment: SpriteAlignment::Billboard,
            });
        }
    }

    /// Spawns a radial burst of fire and smoke particles when an enemy dies.
    pub fn spawn_enemy_explosion(&mut self, pos: Vec3) {
        // 1. Fireball sparks (bursts in all directions)
        let fire_count = self.rng.next_f32_range(16.0, 22.0) as usize;
        for _ in 0..fire_count {
            // Generate a point on a sphere/hemisphere
            let theta = self.rng.next_f32_range(0.0, 6.28);
            let phi = self.rng.next_f32_range(0.0, 3.14); // full sphere

            let vel_dir = Vec3::new(
                phi.sin() * theta.cos(),
                phi.sin() * theta.sin().abs() + 0.2, // bias slightly upwards
                phi.cos(),
            ).normalize();

            let speed = self.rng.next_f32_range(0.3, 0.8);
            let max_lifetime = self.rng.next_f32_range(0.2, 0.4);
            let size = self.rng.next_f32_range(0.015, 0.025);

            // Alternate colors for variety
            let (col_start, col_end) = if self.rng.next_f32() > 0.5 {
                ([1.0, 0.6, 0.1, 1.0], [0.9, 0.1, 0.0, 0.0]) // Orange to red-faded
            } else {
                ([1.0, 0.9, 0.2, 1.0], [0.8, 0.3, 0.0, 0.0]) // Yellow to orange-faded
            };

            self.particles.push(Particle {
                position: pos + vel_dir * 0.01,
                velocity: vel_dir * speed,
                color_start: col_start,
                color_end: col_end,
                size_start: Vec2::new(size, size),
                size_end: Vec2::new(size * 0.1, size * 0.1),
                rotation: self.rng.next_f32_range(0.0, 6.28),
                rotation_speed: self.rng.next_f32_range(-5.0, 5.0),
                lifetime: max_lifetime,
                max_lifetime,
                alignment: SpriteAlignment::Billboard,
            });
        }

        // 2. Thick smoke cloud
        let smoke_count = self.rng.next_f32_range(10.0, 14.0) as usize;
        for _ in 0..smoke_count {
            let theta = self.rng.next_f32_range(0.0, 6.28);
            let vel = Vec3::new(
                theta.cos() * self.rng.next_f32_range(0.06, 0.15),
                self.rng.next_f32_range(0.12, 0.27), // rises upwards
                theta.sin() * self.rng.next_f32_range(0.06, 0.15),
            );

            let max_lifetime = self.rng.next_f32_range(0.5, 0.8);
            let start_size = self.rng.next_f32_range(0.02, 0.035);

            self.particles.push(Particle {
                position: pos + vel * 0.01,
                velocity: vel,
                color_start: [0.4, 0.4, 0.4, 0.7],
                color_end: [0.15, 0.15, 0.15, 0.0],
                size_start: Vec2::new(start_size, start_size),
                size_end: Vec2::new(start_size * 2.2, start_size * 2.2),
                rotation: self.rng.next_f32_range(0.0, 6.28),
                rotation_speed: self.rng.next_f32_range(-1.5, 1.5),
                lifetime: max_lifetime,
                max_lifetime,
                alignment: SpriteAlignment::Billboard,
            });
        }
    }

    /// Spawns a tiny fading trail particle at the projectile's position.
    pub fn spawn_projectile_trail(&mut self, pos: Vec3, color: [f32; 4]) {
        let size = self.rng.next_f32_range(0.005, 0.009);
        let max_lifetime = self.rng.next_f32_range(0.15, 0.25);

        // Slow, tiny puff velocity so the trail lingers in place
        let vel = Vec3::new(
            self.rng.next_f32_range(-0.1, 0.1),
            self.rng.next_f32_range(-0.1, 0.1),
            self.rng.next_f32_range(-0.1, 0.1),
        );

        let mut color_end = color;
        color_end[3] = 0.0; // Fade to transparent

        self.particles.push(Particle {
            position: pos,
            velocity: vel,
            color_start: color,
            color_end,
            size_start: Vec2::new(size, size),
            size_end: Vec2::new(size * 0.1, size * 0.1),
            rotation: self.rng.next_f32_range(0.0, 6.28),
            rotation_speed: self.rng.next_f32_range(-2.0, 2.0),
            lifetime: max_lifetime,
            max_lifetime,
            alignment: SpriteAlignment::Billboard,
        });
    }

    /// Spawns a small impact burst of sparks when a projectile hits a target.
    pub fn spawn_impact_burst(&mut self, pos: Vec3) {
        let spark_count = self.rng.next_f32_range(4.0, 7.0) as usize;
        for _ in 0..spark_count {
            let theta = self.rng.next_f32_range(0.0, 6.28);
            let vel = Vec3::new(
                theta.cos() * self.rng.next_f32_range(0.15, 0.4),
                self.rng.next_f32_range(0.15, 0.4),
                theta.sin() * self.rng.next_f32_range(0.15, 0.4),
            );

            let max_lifetime = self.rng.next_f32_range(0.1, 0.20);
            let size = self.rng.next_f32_range(0.007, 0.012);

            self.particles.push(Particle {
                position: pos,
                velocity: vel,
                color_start: [1.0, 1.0, 0.6, 1.0], // Yellowish-white spark
                color_end: [1.0, 0.2, 0.0, 0.0],
                size_start: Vec2::new(size, size),
                size_end: Vec2::new(size * 0.1, size * 0.1),
                rotation: self.rng.next_f32_range(0.0, 6.28),
                rotation_speed: self.rng.next_f32_range(-5.0, 5.0),
                lifetime: max_lifetime,
                max_lifetime,
                alignment: SpriteAlignment::Billboard,
            });
        }
    }

    /// Clears all particles.
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rng() {
        let mut rng = SimpleRng::new(42);
        let val1 = rng.next_f32();
        let val2 = rng.next_f32();
        assert!(val1 >= 0.0 && val1 <= 1.0);
        assert!(val2 >= 0.0 && val2 <= 1.0);
        assert_ne!(val1, val2);

        let range_val = rng.next_f32_range(10.0, 20.0);
        assert!(range_val >= 10.0 && range_val <= 20.0);
    }

    #[test]
    fn test_particle_update_and_expiry() {
        let mut system = ParticleSystem::new();
        assert_eq!(system.particles.len(), 0);

        // Add a mock particle
        system.particles.push(Particle {
            position: Vec3::ZERO,
            velocity: Vec3::new(1.0, 0.0, 0.0),
            color_start: [1.0, 1.0, 1.0, 1.0],
            color_end: [0.0, 0.0, 0.0, 0.0],
            size_start: Vec2::new(1.0, 1.0),
            size_end: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            rotation_speed: 1.0,
            lifetime: 0.5,
            max_lifetime: 0.5,
            alignment: SpriteAlignment::Billboard,
        });

        assert_eq!(system.particles.len(), 1);

        // Update by 0.1s
        system.update(0.1);
        assert_eq!(system.particles.len(), 1);
        // Position should have changed
        assert!(system.particles[0].position.x > 0.0);
        assert!(system.particles[0].lifetime < 0.5);

        // Update by 0.5s (exceeds lifetime)
        system.update(0.5);
        // Should be removed
        assert_eq!(system.particles.len(), 0);
    }

    #[test]
    fn test_spawning_muzzle_flash() {
        let mut system = ParticleSystem::new();
        system.spawn_muzzle_flash(Vec3::ZERO, Vec2::new(1.0, 0.0));
        // Should spawn both spark and smoke particles
        assert!(system.particles.len() >= 8);
    }

    #[test]
    fn test_spawning_enemy_explosion() {
        let mut system = ParticleSystem::new();
        system.spawn_enemy_explosion(Vec3::ZERO);
        assert!(system.particles.len() >= 25);
    }
}
