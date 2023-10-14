use std::ops::Range;

use wgpu::CommandEncoder;

use crate::app::RenderContext;

pub trait RenderPass {
    fn draw(
        &self,
        render_context: &RenderContext,
        encoder: &mut CommandEncoder,
        verts: Range<u32>,
        instances: Range<u32>,
    );
}

pub trait ComputePass {
    fn exec(&self, render_context: &RenderContext, encoder: &mut CommandEncoder);
}
