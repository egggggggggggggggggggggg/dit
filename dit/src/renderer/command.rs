use crate::renderer::queue::QueueFamiliesIndices;
use crate::renderer::*;
use ash::{Device, vk};
pub fn create_command_pool(
    device: &Device,
    queue_families_indices: QueueFamiliesIndices,
    create_flags: vk::CommandPoolCreateFlags,
) -> vk::CommandPool {
    let command_pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_families_indices.graphics_index)
        .flags(create_flags);

    unsafe {
        device
            .create_command_pool(&command_pool_info, None)
            .unwrap()
    }
}
pub fn create_and_register_command_buffers(
    device: &Device,
    pool: vk::CommandPool,
    framebuffers: &[vk::Framebuffer],
    render_pass: vk::RenderPass,
    swapchain_properties: SwapchainProperties,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: &[vk::DescriptorSet],
    graphics_pipeline: vk::Pipeline,
) -> Vec<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(framebuffers.len() as _);

    let buffers = unsafe { device.allocate_command_buffers(&allocate_info).unwrap() };

    buffers.iter().enumerate().for_each(|(i, buffer)| {
        let buffer = *buffer;
        let framebuffer = framebuffers[i];

        // begin command buffer
        {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
            // .inheritance_info() null since it's a primary command buffer
            unsafe {
                device
                    .begin_command_buffer(buffer, &command_buffer_begin_info)
                    .unwrap()
            };
        }

        // begin render pass
        {
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain_properties.extent,
                })
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                )
            };
        }

        // Bind pipeline
        unsafe {
            device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline)
        };
        //Initial frame vertex buffer (starting so no changes )
        // Bind vertex buffer
        let vertex_buffers = [vertex_buffer];
        let offsets = [0];
        unsafe { device.cmd_bind_vertex_buffers(buffer, 0, &vertex_buffers, &offsets) };

        // Bind index buffer
        unsafe { device.cmd_bind_index_buffer(buffer, index_buffer, 0, vk::IndexType::UINT32) };

        // Bind descriptor set
        unsafe {
            let null = [];
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &descriptor_sets[i..=i],
                &null,
            )
        };
        // Draw
        unsafe { device.cmd_draw_indexed(buffer, index_count as _, 1, 0, 0, 0) };
        // End render pass
        unsafe { device.cmd_end_render_pass(buffer) };
        // End command buffer
        unsafe { device.end_command_buffer(buffer).unwrap() };
    });

    buffers
}
pub fn execute_one_time_commands<F: FnOnce(vk::CommandBuffer)>(
    device: &Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    executor: F,
) {
    let command_buffer = {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        unsafe { device.allocate_command_buffers(&alloc_info).unwrap()[0] }
    };
    let command_buffers = [command_buffer];
    {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap()
        };
    }
    executor(command_buffer);
    unsafe { device.end_command_buffer(command_buffer).unwrap() };
    {
        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
        let submit_infos = [submit_info];
        unsafe {
            device
                .queue_submit(queue, &submit_infos, vk::Fence::null())
                .unwrap();
            device.queue_wait_idle(queue).unwrap();
        };
    }
    unsafe { device.free_command_buffers(command_pool, &command_buffers) };
}

pub fn test(
    device: &Device,
    pool: vk::CommandPool,
    framebuffers: &[vk::Framebuffer],
    render_pass: vk::RenderPass,
    swapchain_properties: SwapchainProperties,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: &[vk::DescriptorSet],
    graphics_pipeline: vk::Pipeline,
    instance_buffer: vk::Buffer,
    instance_count: u32,
) -> Vec<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(framebuffers.len() as _);
    let buffers = unsafe { device.allocate_command_buffers(&allocate_info).unwrap() };
    buffers.iter().enumerate().for_each(|(i, buffer)| {
        let buffer = *buffer;
        let framebuffer = framebuffers[i];
        // begin command buffer
        {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
            // .inheritance_info() null since it's a primary command buffer
            unsafe {
                device
                    .begin_command_buffer(buffer, &command_buffer_begin_info)
                    .unwrap()
            };
        }

        // begin render pass
        {
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain_properties.extent,
                })
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                )
            };
        }

        // Bind pipeline
        unsafe {
            device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline)
        };
        //Initial frame vertex buffer (starting so no changes )
        // Bind vertex buffer
        let vertex_buffers = [vertex_buffer];
        let instance_buffers = [instance_buffer];
        let offsets = [0];
        unsafe {
            device.cmd_bind_vertex_buffers(buffer, 0, &vertex_buffers, &offsets);
            device.cmd_bind_vertex_buffers(buffer, 1, &instance_buffers, &offsets);
        };
        // Bind index buffer
        unsafe { device.cmd_bind_index_buffer(buffer, index_buffer, 0, vk::IndexType::UINT32) };

        // Bind descriptor set
        unsafe {
            let null = [];
            device.cmd_bind_descriptor_sets(
                buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &descriptor_sets[i..=i],
                &null,
            )
        };
        // Draw
        unsafe { device.cmd_draw_indexed(buffer, index_count as _, instance_count, 0, 0, 0) };
        // End render pass
        unsafe { device.cmd_end_render_pass(buffer) };
        // End command buffer
        unsafe { device.end_command_buffer(buffer).unwrap() };
    });

    buffers
}
