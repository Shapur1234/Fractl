use crate::{Camera, ColorType, FractalType};

use std::{num::NonZeroU32, sync::Mutex};

use cgmath::Vector2;
use wgpu::util::DeviceExt;

static INSTANCE: Mutex<Option<WgpuContext>> = Mutex::new(None);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ArgsUniform {
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
        max_iterations: NonZeroU32,
        selected_fractal: FractalType,
        selected_color: ColorType,
    ) -> Self {
        Self {
            screen_size: screen_size.into().map(|x| x.get()).into(),
            view_size: camera.view_size.map(|x| x as f32).into(),
            zoom: camera.zoom.map(|x| x as f32).into(),
            center_pos: camera.center_pos.map(|x| x as f32).into(),
            max_iterations: max_iterations.get(),
            selected_fractal: selected_fractal as u32,
            selected_color: selected_color as u32,
            _padding: 0,
        }
    }
}

struct WgpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    args_buffer: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    storage_buffer: wgpu::Buffer,
    old_buffer_size: u32,
    output_staging_buffer: wgpu::Buffer,
}

impl WgpuContext {
    fn new(buffer_len: u32) -> Self {
        pollster::block_on(Self::new_async(buffer_len))
    }

    fn update(
        &mut self,
        camera: &Camera,
        screen_size: impl Into<Vector2<NonZeroU32>>,
        max_iterations: NonZeroU32,
        selected_fractal: FractalType,
        selected_color: ColorType,
    ) {
        let screen_size = screen_size.into();

        self.queue.write_buffer(
            &self.args_buffer,
            0,
            bytemuck::cast_slice(&[ArgsUniform::new(
                camera,
                screen_size,
                max_iterations,
                selected_fractal,
                selected_color,
            )]),
        );

        let new_buffer_size = screen_size.x.get() * screen_size.y.get();
        if new_buffer_size != self.old_buffer_size {
            self.old_buffer_size = new_buffer_size;

            todo!();
        }
    }

    async fn new_async(buffer_len: u32) -> Self {
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
            label: Some("Storage buffer"),
            size: (buffer_len * (std::mem::size_of::<u32>() as u32)) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output stagin buffer"),
            size: (buffer_len * (std::mem::size_of::<u32>() as u32)) as wgpu::BufferAddress,
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
            old_buffer_size: buffer_len as u32,
            output_staging_buffer,
        }
    }

    fn gpu_compute(&self, local_buffer: &mut [u32]) {
        pollster::block_on(self.gpu_compute_async(local_buffer))
    }

    async fn gpu_compute_async(&self, local_buffer: &mut [u32]) {
        self.queue
            .write_buffer(&self.storage_buffer, 0, bytemuck::cast_slice(local_buffer));

        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(local_buffer.len() as u32, 1, 1);
        }

        command_encoder.copy_buffer_to_buffer(
            &self.storage_buffer,
            0,
            &self.output_staging_buffer,
            0,
            self.storage_buffer.size(),
        );

        self.queue.submit(Some(command_encoder.finish()));

        let buffer_slice = self.output_staging_buffer.slice(..);

        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());

        self.device.poll(wgpu::Maintain::Wait);
        receiver.recv_async().await.unwrap().unwrap();
        {
            let view = buffer_slice.get_mapped_range();
            local_buffer.copy_from_slice(bytemuck::cast_slice(&view));
        }

        self.output_staging_buffer.unmap();
    }
}

pub(crate) fn do_gpu_compute(
    io_buffer: &mut [u32],
    camera: &Camera,
    screen_size: impl Into<Vector2<NonZeroU32>>,
    max_iterations: NonZeroU32,
    selected_fractal: FractalType,
    selected_color: ColorType,
) {
    let screen_size = screen_size.into();
    let buffer_size = screen_size.x.get() * screen_size.y.get();

    assert_eq!(io_buffer.len() as u32, buffer_size);

    let mut instance_lock = INSTANCE.lock().unwrap();

    if instance_lock.is_none() {
        *instance_lock = Some(WgpuContext::new(buffer_size));
    }

    if let Some(context) = &mut *instance_lock {
        context.update(camera, screen_size, max_iterations, selected_fractal, selected_color);
        context.gpu_compute(io_buffer);
    } else {
        unreachable!()
    }
}
