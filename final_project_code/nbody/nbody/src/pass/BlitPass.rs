use std::{ops::Range};

use wgpu::{
    include_wgsl, util::DeviceExt, Buffer, CommandEncoder, ShaderStages,
    TextureViewDescriptor,
};

use crate::{
    app::RenderContext,
    pipelines::{BindgroupBuilder, RenderPipeline, RenderPipelineBuilder, BindingResource, Binding},
};

use super::RenderPass::RenderPass;

pub struct BlitPass {
    pl_blit: RenderPipeline,
    res_unif: Buffer,
    res_internal_unif: Buffer
}

impl BlitPass {
    pub fn new(ctx: &RenderContext) -> Self {
        let final_view = ctx
            .color_target
            .create_view(&TextureViewDescriptor::default());

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let res_unif = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Resolution Uniform"),
                contents: bytemuck::cast_slice(&[0f32; 2]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let res_internal_unif = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Internal Resolution Uniform"),
                contents: bytemuck::cast_slice(&[0f32; 2]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bg = BindgroupBuilder::new()
            .resource( Binding { vis: ShaderStages::FRAGMENT, res: BindingResource::Uniform(&res_unif)})
            .resource( Binding { vis: ShaderStages::FRAGMENT, res: BindingResource::Uniform(&res_internal_unif)})
            .resource( Binding { vis: ShaderStages::FRAGMENT, res: BindingResource::Texture(&final_view, &sampler)});

        Self {
            pl_blit: {
                RenderPipelineBuilder::new()
                    .vert(&ctx.device, include_wgsl!("../../shaders/default.wgsl"))
                    .frag(&ctx.device, include_wgsl!("../../shaders/blit.wgsl"))
                    .bind_group(&ctx.device, bg)
                    .topo(wgpu::PrimitiveTopology::TriangleList)
                    .name("Blit Pipeline")
                    .build(&ctx.device, &ctx.surface_configuration.format)
            },
            res_unif,
            res_internal_unif
        }
    }
}

impl<'surf> RenderPass for BlitPass {
    fn draw(
        &self,
        ctx: &RenderContext,
        encoder: &mut CommandEncoder,
        verts: Range<u32>,
        instances: Range<u32>,
    ) {
        ctx.command_queue.write_buffer(
            &self.res_unif,
            0,
            bytemuck::cast_slice(&[
                ctx.window.inner_size().width as f32,
                ctx.window.inner_size().height as f32,
            ]),
        );

        ctx.command_queue.write_buffer(
            &self.res_internal_unif,
            0,
            bytemuck::cast_slice(&[
                ctx.internal_target_size.0 as f32,
                ctx.internal_target_size.1 as f32,
            ]),
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blit Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.current_surface_texture.as_ref().unwrap(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.pl_blit.bind(&mut render_pass);
        render_pass.draw(verts, instances);
    }
}
