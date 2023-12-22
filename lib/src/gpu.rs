use crate::Camera;

use std::num::NonZeroU32;

use cgmath::Vector2;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ArgsUniform {
    screen_size: [u32; 2],
    view_size: [f32; 2],
    zoom: [f32; 2],
    center_pos: [f32; 2],
    max_iterations: u32,
    selected_fractal: u32,
    selected_color: u32,
    _padding: u32,
}

impl ArgsUniform {
    pub fn new(
        camera: &Camera,
        screen_size: impl Into<Vector2<NonZeroU32>>,
        max_iterations: u32,
        selected_fractal: u32,
        selected_color: u32,
    ) -> Self {
        Self {
            screen_size: screen_size.into().map(|x| x.get()).into(),
            view_size: camera.view_size.map(|x| x as f32).into(),
            zoom: camera.zoom.map(|x| x as f32).into(),
            center_pos: camera.center_pos.map(|x| x as f32).into(),
            max_iterations,
            selected_fractal,
            selected_color,
            _padding: 0,
        }
    }
}

pub struct WgpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    args_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    storage_buffer: wgpu::Buffer,
    output_staging_buffer: wgpu::Buffer,
}

impl WgpuContext {
    pub fn new(buffer_size: usize) -> Self {
        pollster::block_on(Self::new_async(buffer_size))
    }

    pub fn update_args(&mut self, args_uniform: ArgsUniform) {
        self.queue
            .write_buffer(&self.args_buffer, 0, bytemuck::cast_slice(&[args_uniform]));
    }

    async fn new_async(buffer_size: usize) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("./shader.wgsl"))),
        });

        let args_uniform = ArgsUniform::default();
        let args_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Args Buffer"),
            contents: bytemuck::cast_slice(&[args_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: args_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            device,
            queue,
            args_buffer,
            pipeline,
            bind_group,
            storage_buffer,
            output_staging_buffer,
        }
    }
}

pub fn gpu_compute(local_buffer: &mut [u32], args: ArgsUniform, context: &WgpuContext) {
    pollster::block_on(gpu_compute_async(local_buffer, args, context))
}

async fn gpu_compute_async(local_buffer: &mut [u32], args: ArgsUniform, context: &WgpuContext) {
    context
        .queue
        .write_buffer(&context.storage_buffer, 0, bytemuck::cast_slice(local_buffer));

    let mut command_encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&context.pipeline);
        compute_pass.set_bind_group(0, &context.bind_group, &[]);
        compute_pass.dispatch_workgroups(local_buffer.len() as u32, 1, 1);
    }

    command_encoder.copy_buffer_to_buffer(
        &context.storage_buffer,
        0,
        &context.output_staging_buffer,
        0,
        context.storage_buffer.size(),
    );

    context.queue.submit(Some(command_encoder.finish()));

    let buffer_slice = context.output_staging_buffer.slice(..);

    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());

    context.device.poll(wgpu::Maintain::Wait);
    receiver.recv_async().await.unwrap().unwrap();
    {
        let view = buffer_slice.get_mapped_range();
        local_buffer.copy_from_slice(bytemuck::cast_slice(&view));
    }

    context.output_staging_buffer.unmap();
}
