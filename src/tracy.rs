pub(crate) fn create_tracy_gpu_client(
    backend: wgpu::Backend,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    timestamp_period: f32,
) -> tracy_client::GpuContext {
    let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
        label: Some("wgpu-profiler gpu -> cpu sync query_set"),
        ty: wgpu::QueryType::Timestamp,
        count: 1,
    });

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("wgpu-profiler gpu -> cpu sync buffer"),
        size: crate::QUERY_SIZE as _,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("wgpu-profiler gpu -> cpu sync cmd_buf"),
    });
    encoder.write_timestamp(&query_set, 0);
    encoder.resolve_query_set(&query_set, 0..1, &buffer, 0);
    queue.submit(Some(encoder.finish()));

    let _ = buffer.slice(..).map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);

    let view = buffer.slice(..).get_mapped_range();
    let timestamp: i64 = *bytemuck::from_bytes(&view);

    let tracy_backend = match backend {
        wgpu::Backend::Empty | wgpu::Backend::Metal | wgpu::Backend::BrowserWebGpu => tracy_client::GpuContextType::Invalid,
        wgpu::Backend::Vulkan => tracy_client::GpuContextType::Vulkan,
        wgpu::Backend::Dx12 => tracy_client::GpuContextType::Direct3D12,
        wgpu::Backend::Dx11 => tracy_client::GpuContextType::Direct3D11,
        wgpu::Backend::Gl => tracy_client::GpuContextType::OpenGL,
    };

    tracy_client::Client::running()
        .expect("tracy client not running")
        .new_gpu_context(Some("wgpu"), tracy_backend, timestamp, timestamp_period)
}
