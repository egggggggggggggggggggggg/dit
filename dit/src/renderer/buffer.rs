use std::os::raw::c_void;

use crate::renderer::*;
use ash::{Device, util::Align, vk};
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
    create_device_local_buffer_with_data::<u16, _>(
        vk_context,
        command_pool,
        transfer_queue,
        vk::BufferUsageFlags::INDEX_BUFFER,
        indices,
    )
}
#[derive(Copy, Clone)]
pub struct Buffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    memory_size: u64,
}

#[derive(Clone, Copy)]
pub struct DynamicVertexBuffer {
    pub staging: Buffer,
    pub device: Buffer,
    pub capacity: u64,
    pub write_offset: u64,
    pub data_ptr: *mut c_void,
}

impl DynamicVertexBuffer {
    pub fn new<A, T: Copy>(
        vk_context: &VkContext,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue,
        usage: vk::BufferUsageFlags,
        allocation_size: u64,
        data: &[T],
    ) -> Self {
        //pre allocation approach
        let device = vk_context.device();
        let size = size_of_val(data) as vk::DeviceSize;
        if size > allocation_size {
            panic!("Allocated space was less than the size of data");
        }
        let (staging_buffer, staging_memory, staging_mem_size) = create_buffer(
            vk_context,
            allocation_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        //keep it mapped as data must be continously written to it

        let data_ptr = unsafe {
            let data_ptr = device
                .map_memory(
                    staging_memory,
                    0,
                    allocation_size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut align = ash::util::Align::new(data_ptr, align_of::<A>() as _, staging_mem_size);
            align.copy_from_slice(data);
            data_ptr
        };
        let (buffer, memory, memory_size) = create_buffer(
            vk_context,
            allocation_size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        copy_buffer(
            device,
            command_pool,
            transfer_queue,
            staging_buffer,
            buffer,
            allocation_size,
            0,
            0,
        );
        //copies the data over to
        let staging = Buffer {
            buffer: staging_buffer,
            memory: staging_memory,
            memory_size: staging_mem_size,
        };
        let device = Buffer {
            buffer,
            memory,
            memory_size,
        };
        //len might be the wrong approach here as we want the number of bytes not elements
        Self {
            staging,
            device,
            capacity: allocation_size,
            write_offset: size_of_val(data) as u64,
            data_ptr,
        }
        //creates a simple staging buffer for transfer
    }

    pub fn copy_new_data<A, T: Copy>(
        &mut self,
        vk_context: &VkContext,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue,
        new_data: &[T],
    ) -> bool {
        //if the new data to copy is bigger than the allocated size return a bool signal a reallocation
        //the application must then itself determine the appropiate new allocation size
        let write_offset = self.write_offset;
        let size = size_of_val(new_data) as vk::DeviceSize;
        if (size + write_offset) > self.capacity {
            return false;
        }
        let device = vk_context.device();
        let dst = unsafe { (self.data_ptr as *mut u8).add(write_offset as usize) };
        let mut align = unsafe {
            ash::util::Align::new(
                dst as *mut c_void,
                align_of::<A>() as u64,
                self.capacity - write_offset,
            )
        };
        align.copy_from_slice(new_data);
        copy_buffer(
            device,
            command_pool,
            transfer_queue,
            self.staging.buffer,
            self.device.buffer,
            size,
            write_offset,
            write_offset,
        );
        self.write_offset += size as u64;
        true
    }
    //writes new data to an offset that already has data written to it

    pub fn override_data() {}

    pub fn reallocate(&mut self) {}
    pub fn destroy(&self) {}
}

struct BufferAllocator {}
impl BufferAllocator {
    fn new() {}
}

struct Allocation {
    offset: u64,
    size: u64,
}
