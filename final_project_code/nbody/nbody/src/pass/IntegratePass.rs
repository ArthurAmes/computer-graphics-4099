use std::rc::Rc;

use wgpu::{
    include_wgsl, CommandEncoder, ShaderStages,
    TextureViewDescriptor, Buffer, include_spirv_raw, util::DeviceExt,
};

use crate::{
    app::{RenderContext, N_PARTS},
    pipelines::{BindgroupBuilder, ComputePipeline, ComputePipelineBuilder, Binding, BindingResource}, simulation::star::Star,
};

use super::RenderPass::ComputePass;

pub struct IntegratePass {
    update_positions: ComputePipeline,
    stars: Rc<Buffer>
}

impl IntegratePass {
    pub fn new(ctx: &RenderContext, bufs: Rc<Buffer>) -> Self {     
        let np_unif = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Part Count Uniform"),
                contents: bytemuck::cast_slice(&[N_PARTS]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bg = BindgroupBuilder::new()
            .resource( Binding { vis: ShaderStages::COMPUTE, res: BindingResource::Buffer(bufs.as_ref(), false)})
            .resource( Binding { vis: ShaderStages::COMPUTE, res: BindingResource::Uniform(&np_unif)});

        Self {
            update_positions: {
                ComputePipelineBuilder::new(&ctx.device, include_wgsl!("../../shaders/integrate.wgsl"))
                    .bind_group(&ctx.device, bg)
                    .name("Update Positions Pipeline")
                    .build(&ctx.device)
            },
            stars: bufs
        }
    }
}

impl<'surf> ComputePass for IntegratePass {
    fn exec(
        &self,
        ctx: &RenderContext,
        encoder: &mut CommandEncoder
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Integrate Compute Pass")
        });

        self.update_positions.bind(&mut compute_pass);
        compute_pass.dispatch_workgroups((self.stars.size() / std::mem::size_of::<Star>() as u64 / 64) as u32, 1, 1);
    }
}
