use std::{ops::Range, rc::Rc};

use wgpu::{
    include_wgsl, Buffer, CommandEncoder, TextureView,
    TextureViewDescriptor, TextureFormat, util::DeviceExt, ShaderStages,
};

use crate::{
    app::RenderContext,
    pipelines::{RenderPipeline, RenderPipelineBuilder, BindgroupBuilder, Binding, BindingResource},
    simulation::star::Star,
};

use super::RenderPass::RenderPass;

pub struct ColorPass {
    pl_drawstars: RenderPipeline,
    pl_drawgas: RenderPipeline,
    output_view: TextureView,
    vp_buf: Buffer
}

impl ColorPass {
    pub fn new(ctx: &RenderContext, stars: Rc<Buffer>) -> Self {
        let target = ctx
            .color_target
            .create_view(&TextureViewDescriptor::default());

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Star>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32, 2 => Float32x3, 3 => Float32],
        };

        let vp = (ctx.camera_proj * ctx.camera_view);
        let vp_ref: Vec<f32> = vp.as_array().iter().flat_map(|v| *v.as_array()).collect();

        let vp_unif = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("VP Matrix"),
                contents: bytemuck::cast_slice(vp_ref.as_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bg = BindgroupBuilder::new()
            .resource( Binding { vis: ShaderStages::VERTEX, res: BindingResource::Uniform(&vp_unif)});

        Self {
            pl_drawstars: {
                RenderPipelineBuilder::new()
                    .vert(&ctx.device, include_wgsl!("../../shaders/draw_stars.wgsl"))
                    .frag(&ctx.device, include_wgsl!("../../shaders/draw_stars.wgsl"))
                    .vertex_buffer(vertex_buffer_layout, stars)
                    .bind_group(&ctx.device, bg)
                    .topo(wgpu::PrimitiveTopology::PointList)
                    .name("Draw Stars")
                    .build(&ctx.device, &TextureFormat::Rgba8Unorm)
            },
            pl_drawgas: {
                RenderPipelineBuilder::new()
                    .vert(&ctx.device, include_wgsl!("../../shaders/draw_gas.wgsl"))
                    .frag(&ctx.device, include_wgsl!("../../shaders/draw_gas.wgsl"))
                    .build(&ctx.device, &TextureFormat::Rgba8Unorm)
            },
            output_view: target,
            vp_buf: vp_unif
        }
    }
}

impl<'surf> RenderPass for ColorPass {
    fn draw(
        &self,
        ctx: &RenderContext,
        encoder: &mut CommandEncoder,
        verts: Range<u32>,
        instances: Range<u32>,
    ) {
        let vp = (ctx.camera_proj * ctx.camera_view);
        let vp_ref: Vec<f32> = vp.as_array().iter().flat_map(|v| *v.as_array()).collect();

        ctx.command_queue.write_buffer(
            &self.vp_buf,
            0,
            bytemuck::cast_slice(vp_ref.as_slice()),
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // self.pl_drawgas.draw(&mut render_pass);
        self.pl_drawstars.bind(&mut render_pass);
        render_pass.draw(verts, instances);
    }
}
