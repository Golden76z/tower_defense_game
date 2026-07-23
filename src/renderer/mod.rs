pub mod batch;
pub mod camera;
pub mod model;
pub mod sprite;
pub mod texture;

use glam::Mat4;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
    }
}

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    overlay_pipeline: wgpu::RenderPipeline,
    ui_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    depth_texture: texture::Texture,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        camera: &camera::Camera,
    ) -> Self {
        // Load shaders into shader modules
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        // Initialize camera uniform & buffer
        let mut camera_uniform = CameraUniform::new();
        let view_proj = camera.build_view_projection_matrix(config.width, config.height);
        camera_uniform.update_view_proj(view_proj);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Load grass side texture (texture ID 0)
        let grass_side = texture::Texture::load(device, queue, "assets/sprites/grass.png")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load grass side texture: {:?}", e);
                let fallback_img =
                    image::RgbaImage::from_pixel(16, 16, image::Rgba([139, 90, 60, 255]));
                texture::Texture::from_image(
                    device,
                    queue,
                    &image::DynamicImage::ImageRgba8(fallback_img),
                    Some("Fallback Side"),
                )
                .unwrap()
            });

        // Load grass top texture (texture ID 1)
        let grass_top = texture::Texture::load(device, queue, "assets/sprites/grass_top.png")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load grass top texture: {:?}", e);
                let fallback_img =
                    image::RgbaImage::from_pixel(16, 16, image::Rgba([100, 180, 70, 255]));
                texture::Texture::from_image(
                    device,
                    queue,
                    &image::DynamicImage::ImageRgba8(fallback_img),
                    Some("Fallback Top"),
                )
                .unwrap()
            });

        // Load water texture (texture ID 2)
        let water_tex = texture::Texture::load(device, queue, "assets/sprites/water.png")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load water texture: {:?}", e);
                let fallback_img =
                    image::RgbaImage::from_pixel(16, 16, image::Rgba([40, 100, 200, 200])); // Semi-transparent blue
                texture::Texture::from_image(
                    device,
                    queue,
                    &image::DynamicImage::ImageRgba8(fallback_img),
                    Some("Fallback Water"),
                )
                .unwrap()
            });

        // Load sand texture (texture ID 3)
        let sand_tex = texture::Texture::load(device, queue, "assets/sprites/sand.png")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load sand texture: {:?}", e);
                let fallback_img =
                    image::RgbaImage::from_pixel(16, 16, image::Rgba([230, 200, 130, 255]));
                texture::Texture::from_image(
                    device,
                    queue,
                    &image::DynamicImage::ImageRgba8(fallback_img),
                    Some("Fallback Sand"),
                )
                .unwrap()
            });

        // Load rock texture (texture ID 4)
        let rock_tex = texture::Texture::load(device, queue, "assets/sprites/rock.png")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load rock texture: {:?}", e);
                let fallback_img =
                    image::RgbaImage::from_pixel(16, 16, image::Rgba([120, 120, 120, 255]));
                texture::Texture::from_image(
                    device,
                    queue,
                    &image::DynamicImage::ImageRgba8(fallback_img),
                    Some("Fallback Rock"),
                )
                .unwrap()
            });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // Create bind group for grass side (ID 0)
        let side_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&grass_side.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&grass_side.sampler),
                },
            ],
            label: Some("grass_side_bind_group"),
        });

        // Create bind group for grass top (ID 1)
        let top_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&grass_top.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&grass_top.sampler),
                },
            ],
            label: Some("grass_top_bind_group"),
        });

        let water_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&water_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&water_tex.sampler),
                },
            ],
            label: Some("water_bind_group"),
        });

        let sand_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sand_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sand_tex.sampler),
                },
            ],
            label: Some("sand_bind_group"),
        });

        let rock_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&rock_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&rock_tex.sampler),
                },
            ],
            label: Some("rock_bind_group"),
        });

        // Create pipeline layout containing both bind group layouts
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Configure pipeline descriptor and set up render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    // Alpha blending so translucent overlays (e.g. the tile
                    // hover highlight) composite over opaque terrain. Opaque
                    // geometry uses alpha = 1.0, which blends identically to
                    // REPLACE, so terrain is unaffected.
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlay Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let depth_texture = texture::Texture::create_depth_texture(device, config, "depth_texture");

        Self {
            render_pipeline,
            overlay_pipeline,
            ui_pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            texture_bind_groups: vec![
                side_bind_group,
                top_bind_group,
                water_bind_group,
                sand_bind_group,
                rock_bind_group,
            ],
            depth_texture,
        }
    }

    /// Registers a new texture and returns its assigned texture_id.
    pub fn register_texture(&mut self, device: &wgpu::Device, texture: &texture::Texture) -> usize {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("dynamic_texture_bind_group"),
        });

        let id = self.texture_bind_groups.len();
        self.texture_bind_groups.push(bind_group);
        id
    }

    pub fn update_camera_uniform(
        &self,
        queue: &wgpu::Queue,
        camera: &camera::Camera,
        width: u32,
        height: u32,
    ) {
        let view_proj = camera.build_view_projection_matrix(width, height);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(view_proj);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture =
            texture::Texture::create_depth_texture(device, config, "depth_texture");
    }

    /// Renders several batchers in a single pass, clearing once. This lets a
    /// cached static batcher (e.g. terrain) be drawn alongside a per-frame
    /// dynamic batcher (towers, enemies, projectiles) without redundant work.
    /// Depth testing resolves ordering between them.
    pub fn render_batches(
        &self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        clear_color: wgpu::Color,
        batchers: &[&batch::SpriteBatcher],
    ) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Multi-Batch Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Multi-Batch Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for batcher in batchers {
                if let Some(vertex_buffer) = batcher.vertex_buffer() {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    for batch in batcher.batches() {
                        if let Some(bind_group) = self.texture_bind_groups.get(batch.texture_id) {
                            render_pass.set_bind_group(1, bind_group, &[]);
                            render_pass.draw(
                                batch.vertex_offset..(batch.vertex_offset + batch.vertex_count),
                                0..1,
                            );
                        }
                    }
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Renders overlay sprite batchers on top of the existing scene with depth testing but without depth writing.
    pub fn render_overlay_batches(
        &self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batchers: &[&batch::SpriteBatcher],
    ) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Overlay Multi-Batch Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Overlay Multi-Batch Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.overlay_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for batcher in batchers {
                if let Some(vertex_buffer) = batcher.vertex_buffer() {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    for batch in batcher.batches() {
                        if let Some(bind_group) = self.texture_bind_groups.get(batch.texture_id) {
                            render_pass.set_bind_group(1, bind_group, &[]);
                            render_pass.draw(
                                batch.vertex_offset..(batch.vertex_offset + batch.vertex_count),
                                0..1,
                            );
                        }
                    }
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    /// Renders UI sprite batchers on top of the existing scene without depth testing/occlusion.
    pub fn render_ui_batches(
        &self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batchers: &[&batch::SpriteBatcher],
    ) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("UI Multi-Batch Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Multi-Batch Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.ui_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for batcher in batchers {
                if let Some(vertex_buffer) = batcher.vertex_buffer() {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    for batch in batcher.batches() {
                        if let Some(bind_group) = self.texture_bind_groups.get(batch.texture_id) {
                            render_pass.set_bind_group(1, bind_group, &[]);
                            render_pass.draw(
                                batch.vertex_offset..(batch.vertex_offset + batch.vertex_count),
                                0..1,
                            );
                        }
                    }
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
