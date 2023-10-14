use std::rc::Rc;

use egui::{RichText, FontId};
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{include_wgsl, util::DeviceExt, Buffer, CommandEncoder, ShaderStages, TextureView};

use crate::{
    app::{App, EguiRendCtx},
    pipelines::{BindgroupBuilder, RenderPipeline, RenderPipelineBuilder},
};

use super::RenderPass::RenderPass;

pub struct UIPass {
    egui_rp: EguiRendCtx
}

impl UIPass {
    pub fn new(
        device: &wgpu::Device,
        output_surface: Rc<wgpu::Surface>,
        egui_rp: EguiRendCtx
    ) -> Self {
        Self {
            egui_rp: egui_rp
        }
    }
}

impl<'surf> RenderPass for UIPass {
    fn draw(&self, app: &App, output: &TextureView, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });


        // egui
        self.egui_rp.platform.begin_frame();

        let title = RichText::new("Simulation Controls").font(FontId::monospace(16.0));
        egui::Window::new(title).show(&self.egui_rp.platform.context(), |ui| {});

        let full_output = self.egui_rp.platform.end_frame(None);
        let paint_jobs = self
            .egui_rp
            .platform
            .context()
            .tessellate(full_output.shapes);

        let screen_desc = ScreenDescriptor {
            physical_width: app.window.inner_size().width,
            physical_height: app.window.inner_size().height,
            scale_factor: app.window.scale_factor() as f32,
        };
        let tdelta = full_output.textures_delta;
        self.egui_rp
            .rpass
            .add_textures(&app.device, &app.queue, &tdelta)
            .expect("add textures failed");

        self.egui_rp
            .rpass
            .update_buffers(&app.device, &app.queue, &paint_jobs, &screen_desc);

        {

            self.egui_rp
                .rpass
                .execute_with_renderpass(&mut render_pass, &paint_jobs, &screen_desc)
                .unwrap();
        }
    }
}

