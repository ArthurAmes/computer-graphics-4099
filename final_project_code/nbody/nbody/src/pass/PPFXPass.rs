

use wgpu::{
    include_wgsl, CommandEncoder, ShaderStages,
    TextureViewDescriptor,
};

use crate::{
    app::RenderContext,
    pipelines::{BindgroupBuilder, ComputePipeline, ComputePipelineBuilder, Binding, BindingResource},
};

use super::RenderPass::ComputePass;

pub struct PPFXPass {
    bloom_x: ComputePipeline,
    bloom_y: ComputePipeline
}

impl PPFXPass {
    pub fn new(ctx: &RenderContext) -> Self {     
        let ping = ctx
            .color_target
            .create_view(&TextureViewDescriptor::default());

        let pong = ctx
            .pingpong
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

        let bg_x = BindgroupBuilder::new()
            .resource( Binding { vis: ShaderStages::COMPUTE, res: BindingResource::Texture(&ping, &sampler)})
            .resource(Binding { vis: ShaderStages::COMPUTE, res: BindingResource::TextureStore(ctx.pingpong.format(), &pong, wgpu::StorageTextureAccess::WriteOnly)});

        let bg_y = BindgroupBuilder::new()
            .resource( Binding { vis: ShaderStages::COMPUTE, res: BindingResource::Texture(&pong, &sampler)})
            .resource(Binding { vis: ShaderStages::COMPUTE, res: BindingResource::TextureStore(ctx.pingpong.format(), &ping, wgpu::StorageTextureAccess::WriteOnly)});

        Self {
            bloom_x: {
                ComputePipelineBuilder::new(&ctx.device, include_wgsl!("../../shaders/bloom_x.wgsl"))
                    .bind_group(&ctx.device, bg_x)
                    .name("Bloom X Pipeline")
                    .build(&ctx.device)
            },
            bloom_y: {
                ComputePipelineBuilder::new(&ctx.device, include_wgsl!("../../shaders/bloom_y.wgsl"))
                    .bind_group(&ctx.device, bg_y)
                    .name("Bloom Y Pipeline")
                    .build(&ctx.device)
            }
        }
    }
}

impl<'surf> ComputePass for PPFXPass {
    fn exec(
        &self,
        ctx: &RenderContext,
        encoder: &mut CommandEncoder
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Bloom Compute Pass")
        });

        self.bloom_x.bind(&mut compute_pass);
        compute_pass.dispatch_workgroups(ctx.internal_target_size.0 / 16.0 as u32, ctx.internal_target_size.1 / 16.0 as u32, 1);
        self.bloom_y.bind(&mut compute_pass);
        compute_pass.dispatch_workgroups(ctx.internal_target_size.0 / 16.0 as u32, ctx.internal_target_size.1 / 16.0 as u32, 1);
    }
}
