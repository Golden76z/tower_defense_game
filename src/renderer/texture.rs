use anyhow::{Context, Result};
use image::GenericImageView;
use std::path::Path;

pub struct Texture {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)
            .context("Failed to load image from bytes")?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
    ) -> Result<Self> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref)
            .with_context(|| format!("Failed to read texture file: {:?}", path_ref))?;
        Self::from_bytes(
            device,
            queue,
            &bytes,
            path_ref.to_str().unwrap_or("Loaded Texture"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_image_if_needed() {
        let path = Path::new("assets/sprites/test.png");
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create assets directory");
            }
            // Create a 32x32 red square
            let imgbuf = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
            imgbuf.save(path).expect("Failed to save test image");
        }
    }

    #[test]
    fn test_create_test_image() {
        create_test_image_if_needed();
        let path = Path::new("assets/sprites/test.png");
        assert!(path.exists());
        let img = image::open(path).expect("Failed to open generated image");
        assert_eq!(img.dimensions(), (32, 32));
    }

    #[test]
    fn test_invalid_image_bytes() {
        let invalid_bytes = [0, 1, 2, 3, 4];
        let result = image::load_from_memory(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_file_error() {
        let path = Path::new("assets/sprites/nonexistent_file_xyz.png");
        let result = std::fs::read(path);
        assert!(result.is_err());
    }
}
