use crate::renderer::{context::VkContext, memory::find_memory_type};
use ash::vk;
fn create_buffer(
    vk_context: &VkContext,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    mem_properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
    println!("segfault happened here!!! with usage {:?}", usage);
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
struct DynamicVertexBuffer {
    staging: vk::Buffer,
    device: vk::Buffer,
    capacity: usize,
    write_offset: usize,
}
impl DynamicVertexBuffer {
    fn new<A, T: Copy>(
        vk_context: &VkContext,
        command_pool: vk::CommandPool,
        transfer_queue: vk::Queue,
        usage: vk::BufferUsageFlags,
        allocation_size: u64,
        data: &[T],
    ) {
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
        unsafe {
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
        };
        //creates a simple staging buffer for transfer
    }
}
