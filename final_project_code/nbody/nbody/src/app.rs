use std::{iter, rc::Rc};

use egui_winit_platform::{Platform, PlatformDescriptor};
use glm::{Matrix4, ext::{perspective, translate, look_at_rh}, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, ComputePipeline, Texture, TextureDescriptor, TextureDimension, TextureView, TextureFormat,
};
use winit::window::Window;

use crate::{
    pass::{ColorPass::ColorPass, PPFXPass::PPFXPass, RenderPass::{RenderPass, ComputePass}, BlitPass::BlitPass, IntegratePass::IntegratePass},
    simulation::star::Star,
};

pub const N_PARTS: u32 = 96304;
// pub const N_PARTS: u32 = 64;

pub struct EguiRendCtx {
    pub platform: Platform,
    pub rpass: egui_wgpu_backend::RenderPass,
    pub ctx: egui::Context,
}

struct RenderPasses {
    color_pass: ColorPass,
    ppfx_pass: PPFXPass,
    blit_pass: BlitPass,
    integrate: IntegratePass
}

pub struct RenderContext {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub command_queue: wgpu::Queue,
    pub window: Window,
    pub current_surface_texture: Option<TextureView>,
    pub color_target: Texture,
    pub pingpong: Texture,
    pub internal_target_size: (u32, u32),
    pub camera_proj: Matrix4<f32>,
    pub camera_view: Matrix4<f32>,
    pub frame_cnt: f32,
    pub zoom: f32
}

pub struct Buffers {
    pub star_buffer: Rc<Buffer>,
}

pub struct App {
    pub render_ctx: RenderContext,
    pub bufs: Buffers,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub egui_rp: EguiRendCtx,
    render_passes: RenderPasses,
}

fn update_camera_matrix(aspect: f32) -> Matrix4<f32> {
    perspective(2.0*3.1415 / 5.0, aspect, 0.1, 100.0)
}

impl App {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    // limits: if cfg!(target_arch = "wasm32") {
                    //     wgpu::Limits::downlevel_webgl2_defaults()
                    // } else {
                    //     wgpu::Limits::default()
                    // },
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // Set up uniforms (resolution, framecount, etc)

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "jbm".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/JetBrainsMono-Regular.ttf")),
        );

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "jbm".to_owned());

        // egui
        let platform = Platform::new(PlatformDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor(),
            font_definitions: fonts,
            style: Default::default(),
        });

        let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);

        let egui_rp = EguiRendCtx {
            platform,
            rpass: egui_rpass,
            ctx: egui::Context::default(),
        };

        let internal_target_size = (
            (window.inner_size().width as f32/64.0).ceil() as u32 * 64,
            (window.inner_size().height as f32/64.0).ceil() as u32 * 64
        );

        // render targets
        let color_target = device.create_texture(&TextureDescriptor {
            label: Some("Color Render Target"),
            size: wgpu::Extent3d {
                width: internal_target_size.0,
                height: internal_target_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        // render targets
        let final_target = device.create_texture(&TextureDescriptor {
            label: Some("Final Render Target"),
            size: wgpu::Extent3d {
                width: internal_target_size.0,
                height: internal_target_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let render_context = RenderContext {
            device,
            surface,
            surface_configuration: config,
            command_queue: queue,
            window,
            current_surface_texture: None,
            color_target,
            pingpong: final_target,
            internal_target_size,
            camera_proj: update_camera_matrix(size.width as f32/size.height as f32),
            camera_view: look_at_rh(Vector3{ x: 10.0, y: 0.0, z: 0.0}, Vector3{ x: 0.0, y: 0.0, z: 0.0},  Vector3{ x: 0.0, y: 0.0, z: 1.0}),
            frame_cnt: 0.0,
            zoom: 5.0
        };

        let mut stars_temp: Vec<Star> = vec![];

        for _ in 0..N_PARTS/2 {
            let x = rand::random::<f32>();
            let y = rand::random::<f32>();
            let z = rand::random::<f32>();

            stars_temp.push(
                Star { 
                    x: x * 9E8 - 10E8,
                    z: z * 9E8, 
                    y: y * 9E8, 
                    x_vel: 0.0,
                    y_vel: 0.0, 
                    z_vel: 0.0, 
                    mass: rand::random::<f32>() * 5E29, // 2E26 = 100 * mass of sun in millions of kg
                    bright: 1.0
                });
        }

        for _ in 0..N_PARTS/2 {
            let x = rand::random::<f32>();
            let y = rand::random::<f32>();
            let z = rand::random::<f32>();

            stars_temp.push(
                Star { 
                    x: x * 9E8 + 10E8,
                    z: z * 9E8, 
                    y: y * 9E8, 
                    x_vel: 0.0,
                    y_vel: 0.0, 
                    z_vel: 0.0, 
                    mass: rand::random::<f32>() * 5E29, // 2E26 = 100 * mass of sun in millions of kg
                    bright: 0.5
                });
        }

        let bufs =
            Buffers {
                star_buffer: Rc::new(render_context.device.create_buffer_init(
                    &BufferInitDescriptor {
                        label: Some("Star Buffer"),
                        contents: bytemuck::cast_slice(stars_temp.as_slice()),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE
                    },
                )),
            };

        let render_passes = RenderPasses {
            color_pass: ColorPass::new(&render_context, bufs.star_buffer.clone()),
            ppfx_pass: PPFXPass::new(&render_context),
            blit_pass: BlitPass::new(&render_context),
            integrate: IntegratePass::new(&render_context, bufs.star_buffer.clone())
        };

        let compute_pipelines: Vec<ComputePipeline> = vec![];

        Self {
            render_ctx: render_context,
            bufs,
            size,
            egui_rp,
            render_passes,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.render_ctx.surface_configuration.width = new_size.width;
            self.render_ctx.surface_configuration.height = new_size.height;
            self.render_ctx.surface.configure(
                &self.render_ctx.device,
                &self.render_ctx.surface_configuration,
            );
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.render_ctx.frame_cnt += 1.0;
        let a = self.render_ctx.frame_cnt / 100.0;
        let z = self.render_ctx.zoom;
        self.render_ctx.camera_view = look_at_rh(Vector3{ x: z * f32::cos(a), y: z * f32::sin(a), z: 0.0}, Vector3{ x: 0.0, y: 0.0, z: 0.0},  Vector3{ x: 0.0, y: 0.0, z: 1.0});

        let output = self.render_ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_ctx.current_surface_texture = Some(view);

        let mut encoder =
            self.render_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        self.render_passes
            .integrate
            .exec(&self.render_ctx, &mut encoder);

        self.render_passes
            .color_pass
            .draw(&self.render_ctx, &mut encoder, 0..N_PARTS, 0..1);

        self.render_passes
            .ppfx_pass
            .exec(&self.render_ctx, &mut encoder);

        self.render_passes
            .blit_pass
            .draw(&self.render_ctx, &mut encoder, 0..6, 0..1);

        self.render_ctx
            .command_queue
            .submit(iter::once(encoder.finish()));

        output.present();

        // self.egui_rp
        //     .rpass
        //     .remove_textures(tdelta)
        //     .expect("failed to remove textures");

        Ok(())
    }
}
