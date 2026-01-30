use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    os::raw::c_void,
};

use crate::renderer::*;
use ash::{Device, vk};
pub fn create_buffer(
    vk_context: &VkContext,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    mem_properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
    let device = vk_context.device();
    let buffer = {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        unsafe { device.create_buffer(&buffer_info, None).unwrap() }
    };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory = {
        let mem_type = find_memory_type(
            mem_requirements,
            vk_context.get_mem_properties(),
            mem_properties,
        );
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type);
        unsafe { device.allocate_memory(&alloc_info, None).unwrap() }
    };
    unsafe { device.bind_buffer_memory(buffer, memory, 0).unwrap() };
    (buffer, memory, mem_requirements.size)
}
pub fn copy_buffer(
    device: &Device,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    src: vk::Buffer,
    dst: vk::Buffer,
    size: vk::DeviceSize,
    src_offset: u64,
    dst_offset: u64,
) {
    execute_one_time_commands(device, command_pool, transfer_queue, |buffer| {
        let region = vk::BufferCopy {
            src_offset,
            dst_offset,
            size,
        };
        let regions = [region];

        unsafe { device.cmd_copy_buffer(buffer, src, dst, &regions) };
    });
}
pub fn copy_buffer_to_image(
    device: &Device,
    command_pool: vk::CommandPool,
    transition_queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    extent: vk::Extent2D,
) {
    execute_one_time_commands(device, command_pool, transition_queue, |command_buffer| {
        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            });
        let regions = [region];
        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            )
        }
    })
}
fn create_device_local_buffer_with_data<A, T: Copy>(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> (vk::Buffer, vk::DeviceMemory) {
    let device = vk_context.device();
    let size = size_of_val(data) as vk::DeviceSize;
    let (staging_buffer, staging_memory, staging_mem_size) = create_buffer(
        vk_context,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = device
            .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())
            .unwrap();

        let mut align = ash::util::Align::new(data_ptr, align_of::<A>() as _, staging_mem_size);
        align.copy_from_slice(data);
        device.unmap_memory(staging_memory);
    };

    let (buffer, memory, _) = create_buffer(
        vk_context,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | usage,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    copy_buffer(
        device,
        command_pool,
        transfer_queue,
        staging_buffer,
        buffer,
        size,
        0,
        0,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);
    };

    (buffer, memory)
}
pub fn create_vertex_buffer(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    vertices: &[Vertex],
) -> (vk::Buffer, vk::DeviceMemory) {
    create_device_local_buffer_with_data::<u32, _>(
        vk_context,
        command_pool,
        transfer_queue,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vertices,
    )
}
pub fn create_index_buffer(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    transfer_queue: vk::Queue,
    indices: &[u32],
) -> (vk::Buffer, vk::DeviceMemory) {
    create_device_local_buffer_with_data::<u32, _>(
        vk_context,
        command_pool,
        transfer_queue,
        vk::BufferUsageFlags::INDEX_BUFFER,
        indices,
    )
}

#[derive(Copy, Clone)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub memory_size: vk::DeviceSize,
}
impl Buffer {
    pub fn new(
        vk_context: &VkContext,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let device = vk_context.device();
        let buffer = {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            unsafe { device.create_buffer(&buffer_info, None).unwrap() }
        };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory = {
            let mem_type = find_memory_type(
                mem_requirements,
                vk_context.get_mem_properties(),
                mem_properties,
            );
            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(mem_type);
            unsafe { device.allocate_memory(&alloc_info, None).unwrap() }
        };
        unsafe { device.bind_buffer_memory(buffer, memory, 0).unwrap() };
        Self {
            buffer,
            memory,
            memory_size: mem_requirements.size,
        }
    }
}
type Offset = u64;
pub struct DynamicBuffer {
    pub allocation_table: HashMap<Offset, Allocation>,
    pub device: Buffer,
    pub staging: Buffer,
    pub data_ptr: *mut c_void,
}

impl DynamicBuffer {
    pub fn new(allocation_size: u64, vk_context: &VkContext, usage: vk::BufferUsageFlags) -> Self {
        let device = vk_context.device();
        let staging = Buffer::new(
            vk_context,
            allocation_size,
            usage | vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let data_ptr = unsafe {
            device
                .map_memory(
                    staging.memory,
                    0,
                    allocation_size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap()
        };
        let device = Buffer::new(
            vk_context,
            allocation_size,
            usage | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        Self {
            allocation_table: HashMap::new(),
            device,
            staging,
            data_ptr,
        }
    }
    pub fn full_copy<A, T: Copy + Debug>(
        &mut self,
        vk_context: &VkContext,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue,
        data: &[T],
    ) {
        let device = vk_context.device();
        unsafe {
            let mut align = ash::util::Align::new(
                self.data_ptr,
                align_of::<A>() as _,
                self.staging.memory_size,
            );
            align.copy_from_slice(data);
        }
        copy_buffer(
            device,
            command_pool,
            transfer_queue,
            self.staging.buffer,
            self.device.buffer,
            size_of_val(data) as vk::DeviceSize,
            0,
            0,
        );
    }
    fn resize() {}
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.unmap_memory(self.device.memory);
            device.destroy_buffer(self.device.buffer, None);
            device.destroy_buffer(self.staging.buffer, None);
            device.free_memory(self.staging.memory, None);
            device.free_memory(self.device.memory, None);
        }
    }
}

struct Allocation {
    offset: u64,
    size: u64,
}
//copies from a given buffer to another one
//staging buffer - allows us to write to it via mapping
//device buffer - thing in gpu memory (fastest for gpu access and use)
//the staging buffer can be written to while the device buffer is in use
//when we have access to the device buffer we can transfer over the data from the staging buffer
//to the device buffer. device buffer then continues using that
//if needed to avoid latency we can have two of these setups so one can consistently stay on the cpu
//and the other on the gpu where we dont have to wait
//for even more speed ups we can query if the device has a dedicated transfer queue for faster speeds
//for manual memory management on the cpu side we must maintain an allocation table specifying the space that is free
//while where we write memory to in the staging buffer doesnt matter, it matters that we keep track of data
//incase fragmentation ever occurs or we need to reallocate a new buffer due to growing storage demand
//persistent mapping of the staging buffer is fine hence the need to store the data ptr in the struct for access
//synchronization will not be managed by the buffer itself, this is up to the user to manually set it up

fn copy_between_buffers() {}
