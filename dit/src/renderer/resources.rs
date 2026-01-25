use ash::{Device, vk};

use crate::renderer::{buffer::create_buffer, context::VkContext, texture::Texture};

#[derive(Clone, Copy)]
#[repr(C)]
struct UniformBufferObject {
    color: [u8; 4],
}

impl UniformBufferObject {
    fn get_descriptor_set_layout_binding<'a>() -> vk::DescriptorSetLayoutBinding<'a> {
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
        // .immutable_samplers() null since we're not creating a sampler descriptor
    }
}

pub fn create_uniform_buffers(
    vk_context: &VkContext,
    count: usize,
) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
    let size = size_of::<UniformBufferObject>() as vk::DeviceSize;
    let mut buffers = Vec::new();
    let mut memories = Vec::new();
    for _ in 0..count {
        let (buffer, memory, _) = create_buffer(
            vk_context,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        buffers.push(buffer);
        memories.push(memory);
    }

    (buffers, memories)
}
pub fn create_descriptor_sets(
    device: &Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    uniform_buffers: &[vk::Buffer],
    texture: Texture,
) -> Vec<vk::DescriptorSet> {
    let layouts = (0..uniform_buffers.len())
        .map(|_| layout)
        .collect::<Vec<_>>();
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() };

    descriptor_sets
        .iter()
        .zip(uniform_buffers.iter())
        .for_each(|(set, buffer)| {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(size_of::<UniformBufferObject>() as vk::DeviceSize);
            let buffer_infos = [buffer_info];

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.view)
                .sampler(texture.sampler.unwrap());
            let image_infos = [image_info];

            let ubo_descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos);
            let sampler_descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos);

            let descriptor_writes = [ubo_descriptor_write, sampler_descriptor_write];

            unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) }
        });

    descriptor_sets
}
pub fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let ubo_binding = UniformBufferObject::get_descriptor_set_layout_binding();
    let sampler_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(1)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let bindings = [sampler_binding, ubo_binding];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap()
    }
}
pub fn create_descriptor_pool(device: &Device, size: u32) -> vk::DescriptorPool {
    let ubo_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: size,
    };
    let sampler_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: size,
    };
    let pool_sizes = [ubo_pool_size, sampler_pool_size];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&pool_sizes)
        .max_sets(size);
    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
}
