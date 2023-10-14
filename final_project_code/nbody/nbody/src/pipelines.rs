use std::rc::Rc;

use wgpu::{
    include_wgsl, BindGroup, BindGroupLayout, Buffer, ComputePass, Device, PrimitiveTopology, RenderPass, Sampler, ShaderModule,
    ShaderModuleDescriptor, TextureFormat, VertexBufferLayout, StorageTextureAccess, ShaderModuleDescriptorSpirV,
};

pub struct RenderPipeline {
    pl: wgpu::RenderPipeline,
    bgs: Vec<wgpu::BindGroup>,
    vertex_buffers: Vec<Rc<Buffer>>,
    samplers: Vec<Sampler>,
}

impl RenderPipeline {
    pub fn bind<'a>(&'a self, rp: &mut RenderPass<'a>) {
        rp.set_pipeline(&self.pl);

        for (i, bg) in self.bgs.iter().enumerate() {
            rp.set_bind_group(i as u32, bg, &[]);
        }

        for (i, vb) in self.vertex_buffers.iter().enumerate() {
            rp.set_vertex_buffer(i as u32, vb.slice(..))
        }
    }
}

pub enum BindingResource<'a> {
    Buffer(&'a wgpu::Buffer, bool),
    Texture(&'a wgpu::TextureView, &'a Sampler),
    Uniform(&'a wgpu::Buffer),
    TextureStore(TextureFormat, &'a wgpu::TextureView, StorageTextureAccess)
}

pub struct Binding<'a> {
    pub vis: wgpu::ShaderStages,
    pub res: BindingResource<'a>,
}

pub struct BindgroupBuilder<'a> {
    resources: Vec<Binding<'a>>,
}

impl<'a> BindgroupBuilder<'a> {
    pub fn new() -> Self {
        Self { resources: vec![] }
    }

    pub fn resource(mut self, res: Binding<'a>) -> Self {
        self.resources.push(res);
        self
    }

    pub fn build(self, device: &Device) -> (BindGroup, BindGroupLayout) {
        let mut layout_entries = vec![];
        let mut group_entries = vec![];

        // Bind Resources
        let mut bi = 0;
        for (_i, res) in self.resources.iter().enumerate() {
            match res.res {
                BindingResource::TextureStore(format, t, access) => {
                    layout_entries.extend([
                        wgpu::BindGroupLayoutEntry {
                            binding: bi as u32,
                            visibility: res.vis,
                            ty: wgpu::BindingType::StorageTexture {
                                access, 
                                format, 
                                view_dimension: wgpu::TextureViewDimension::D2 
                            },
                            count: None,
                        }
                    ]);

                    group_entries.extend([
                        wgpu::BindGroupEntry {
                            binding: bi as u32,
                            resource: wgpu::BindingResource::TextureView(t)
                        }
                    ]);

                    bi += 2;
                }

                BindingResource::Uniform(u) => {
                    layout_entries.push(wgpu::BindGroupLayoutEntry {
                        binding: bi as u32,
                        visibility: res.vis,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    });

                    group_entries.push(wgpu::BindGroupEntry {
                        binding: bi as u32,
                        resource: u.as_entire_binding(),
                    });

                    bi += 1;
                }

                BindingResource::Buffer(b, ro) => {
                    layout_entries.extend([wgpu::BindGroupLayoutEntry {
                        binding: bi as u32,
                        visibility: res.vis,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: ro },
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        },
                        count: None,
                    }]);

                    group_entries.extend([
                        wgpu::BindGroupEntry {
                            binding: bi as u32,
                            resource: b.as_entire_binding()
                        }
                    ]);

                    bi += 1;
                }

                BindingResource::Texture(t, s) => {
                    layout_entries.extend([
                        wgpu::BindGroupLayoutEntry {
                            binding: bi as u32,
                            visibility: res.vis,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: (bi + 1) as u32,
                            visibility: res.vis,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ]);

                    group_entries.extend([
                        wgpu::BindGroupEntry {
                            binding: bi as u32,
                            resource: wgpu::BindingResource::TextureView(t)
                        },
                        wgpu::BindGroupEntry {
                            binding: (bi+1) as u32,
                            resource: wgpu::BindingResource::Sampler(s)
                        }
                    ]);

                    bi += 2;
                }
            }
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layout_entries,
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &group_entries,
            label: None,
        });

        (bind_group, bind_group_layout)
    }
}

pub struct RenderPipelineBuilder<'a> {
    bg_layouts: Vec<BindGroupLayout>,
    bgs: Vec<BindGroup>,
    frag: Option<ShaderModule>,
    vert: Option<ShaderModule>,
    vertex_buffers: Vec<Rc<Buffer>>,
    vertex_buffer_layouts: Vec<VertexBufferLayout<'a>>,
    topo: wgpu::PrimitiveTopology,
    name: &'a str
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            bg_layouts: vec![],
            bgs: vec![],
            frag: None,
            vert: None,
            vertex_buffers: vec![],
            vertex_buffer_layouts: vec![],
            topo: wgpu::PrimitiveTopology::TriangleList,
            name: "Render Pipeline"
        }
    }

    pub fn bind_group(mut self, device: &Device, builder: BindgroupBuilder) -> Self {
        let (bg, layout) = builder.build(device);

        self.bg_layouts.push(layout);
        self.bgs.push(bg);

        self
    }

    pub fn frag(mut self, device: &Device, module: ShaderModuleDescriptor) -> Self {
        self.frag = Some(device.create_shader_module(module));
        self
    }

    pub fn vert(mut self, device: &Device, module: ShaderModuleDescriptor) -> Self {
        self.vert = Some(device.create_shader_module(module));
        self
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn topo(mut self, topo: PrimitiveTopology) -> Self {
        self.topo = topo;
        self
    }

    pub fn vertex_buffer(mut self, layout: VertexBufferLayout<'a>, buffer: Rc<Buffer>) -> Self {
        self.vertex_buffer_layouts.push(layout);
        self.vertex_buffers.push(buffer);
        self
    }

    pub fn build(self, device: &Device, format: &TextureFormat) -> RenderPipeline {
        let frag = self
            .frag
            .unwrap_or(device.create_shader_module(include_wgsl!("../shaders/default.wgsl")));
        let vert = self
            .vert
            .unwrap_or(device.create_shader_module(include_wgsl!("../shaders/default.wgsl")));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.name),
            layout: {
                Some({
                    let mut refs = vec![];
                    for i in 0..self.bg_layouts.len() {
                        refs.push(&self.bg_layouts[i]);
                    }

                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &refs,
                        push_constant_ranges: &[],
                    })
                })
            },
            vertex: wgpu::VertexState {
                module: &vert,
                entry_point: "vs_main",               // 1.
                buffers: &self.vertex_buffer_layouts, // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &frag,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: *format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: self.topo, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        RenderPipeline {
            pl: render_pipeline,
            bgs: self.bgs,
            vertex_buffers: self.vertex_buffers,
            samplers: vec![],
        }
    }
}

pub struct ComputePipelineBuilder<'a> {
    bg_layouts: Vec<BindGroupLayout>,
    bgs: Vec<BindGroup>,
    shader: ShaderModule,
    vertex_buffers: Vec<Rc<Buffer>>,
    vertex_buffer_layouts: Vec<VertexBufferLayout<'a>>,
    topo: wgpu::PrimitiveTopology,
    name: &'a str
}

pub struct ComputePipeline {
    pl: wgpu::ComputePipeline,
    bgs: Vec<wgpu::BindGroup>,
    // samplers: Vec<Sampler>,
}

impl ComputePipeline {
    pub fn bind<'a>(&'a self, rp: &mut ComputePass<'a>) {
        rp.set_pipeline(&self.pl);

        for (i, bg) in self.bgs.iter().enumerate() {
            rp.set_bind_group(i as u32, bg, &[]);
        }
    }
}

impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(device: &Device, module: ShaderModuleDescriptor) -> Self {
        Self {
            bg_layouts: vec![],
            bgs: vec![],
            shader: device.create_shader_module(module),
            vertex_buffers: vec![],
            vertex_buffer_layouts: vec![],
            topo: wgpu::PrimitiveTopology::TriangleList,
            name: "Compute Pipeline"
        }
    }

    pub fn new_spv(device: &Device, module: ShaderModuleDescriptorSpirV) -> Self {
        Self {
            bg_layouts: vec![],
            bgs: vec![],
            shader: unsafe { device.create_shader_module_spirv(&module) },
            vertex_buffers: vec![],
            vertex_buffer_layouts: vec![],
            topo: wgpu::PrimitiveTopology::TriangleList,
            name: "Compute Pipeline"
        }
    }

    pub fn bind_group(mut self, device: &Device, builder: BindgroupBuilder) -> Self {
        let (bg, layout) = builder.build(device);

        self.bg_layouts.push(layout);
        self.bgs.push(bg);

        self
    }

    pub fn topo(mut self, topo: PrimitiveTopology) -> Self {
        self.topo = topo;
        self
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn vertex_buffer(mut self, layout: VertexBufferLayout<'a>, buffer: Rc<Buffer>) -> Self {
        self.vertex_buffer_layouts.push(layout);
        self.vertex_buffers.push(buffer);
        self
    }

    pub fn build(self, device: &Device) -> ComputePipeline {
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(self.name),
            layout: Some({
                let mut refs = vec![];
                for i in 0..self.bg_layouts.len() {
                    refs.push(&self.bg_layouts[i]);
                }

                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &refs,
                    push_constant_ranges: &[],
                })
            }),
            module: &self.shader,
            entry_point: "cs_main",
        });

        ComputePipeline {
            pl: compute_pipeline,
            bgs: self.bgs,
        }
    }
}
