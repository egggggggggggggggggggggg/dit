use crate::renderer::utils::load;
use crate::renderer::*;
use ash::{Device, vk};
pub fn create_texture_image(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    copy_queue: vk::Queue,
) -> Texture {
    let cursor = load("texture_atlas.png");
    let image = image::load(cursor, image::ImageFormat::Png).unwrap();
    let image_as_rgb = image.to_rgba8();
    let width = image_as_rgb.width();
    let height = image_as_rgb.height();
    let max_mip_levels = ((width.min(height) as f32).log2().floor() + 1.0) as u32;
    let extent = vk::Extent2D { width, height };
    let pixels = image_as_rgb.into_raw();
    let image_size = (pixels.len() * size_of::<u8>()) as vk::DeviceSize;
    let device = vk_context.device();

    let (buffer, memory, mem_size) = create_buffer(
        vk_context,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let ptr = device
            .map_memory(memory, 0, image_size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut align = ash::util::Align::new(ptr, align_of::<u8>() as _, mem_size);
        align.copy_from_slice(&pixels);
        device.unmap_memory(memory);
    }

    let (image, image_memory) = create_image(
        vk_context,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        extent,
        max_mip_levels,
        vk::SampleCountFlags::TYPE_1,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED,
    );

    // Transition the image layout and copy the buffer into the image
    // and transition the layout again to be readable from fragment shader.
    {
        transition_image_layout(
            device,
            command_pool,
            copy_queue,
            image,
            max_mip_levels,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        copy_buffer_to_image(device, command_pool, copy_queue, buffer, image, extent);
    }

    unsafe {
        device.destroy_buffer(buffer, None);
        device.free_memory(memory, None);
    }

    let image_view = create_image_view(
        device,
        image,
        max_mip_levels,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageAspectFlags::COLOR,
    );

    let sampler = {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(max_mip_levels as _);

        unsafe { device.create_sampler(&sampler_info, None).unwrap() }
    };

    Texture::new(image, image_memory, image_view, Some(sampler))
}
pub fn create_color_texture(
    vk_context: &VkContext,
    command_pool: vk::CommandPool,
    transition_queue: vk::Queue,
    swapchain_properties: SwapchainProperties,
    msaa_samples: vk::SampleCountFlags,
) -> Texture {
    let format = swapchain_properties.format.format;
    let (image, memory) = create_image(
        vk_context,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        swapchain_properties.extent,
        1,
        msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
    );

    transition_image_layout(
        vk_context.device(),
        command_pool,
        transition_queue,
        image,
        1,
        format,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    );

    let view = create_image_view(
        vk_context.device(),
        image,
        1,
        format,
        vk::ImageAspectFlags::COLOR,
    );

    Texture::new(image, memory, view, None)
}
pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    mip_levels: u32,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        });

    unsafe { device.create_image_view(&create_info, None).unwrap() }
}
fn create_image(
    vk_context: &VkContext,
    mem_properties: vk::MemoryPropertyFlags,
    extent: vk::Extent2D,
    mip_levels: u32,
    sample_count: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(sample_count)
        .flags(vk::ImageCreateFlags::empty());

    let device = vk_context.device();
    let image = unsafe { device.create_image(&image_info, None).unwrap() };
    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let mem_type_index = find_memory_type(
        mem_requirements,
        vk_context.get_mem_properties(),
        mem_properties,
    );

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index);
    let memory = unsafe {
        let mem = device.allocate_memory(&alloc_info, None).unwrap();
        device.bind_image_memory(image, mem, 0).unwrap();
        mem
    };

    (image, memory)
}
fn transition_image_layout(
    device: &Device,
    command_pool: vk::CommandPool,
    transition_queue: vk::Queue,
    image: vk::Image,
    mip_levels: u32,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    execute_one_time_commands(device, command_pool, transition_queue, |buffer| {
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                ) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
                    (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    )
                }
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ),
                _ => panic!(
                    "Unsupported layout transition({:?} => {:?}).",
                    old_layout, new_layout
                ),
            };

        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            let mut mask = vk::ImageAspectFlags::DEPTH;
            if format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT {
                mask |= vk::ImageAspectFlags::STENCIL;
            }
            mask
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);
        let barriers = [barrier];

        unsafe {
            device.cmd_pipeline_barrier(
                buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            )
        };
    });
}

#[derive(Clone, Copy)]
pub struct Texture {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub sampler: Option<vk::Sampler>,
}

impl Texture {
    pub fn new(
        image: vk::Image,
        memory: vk::DeviceMemory,
        view: vk::ImageView,
        sampler: Option<vk::Sampler>,
    ) -> Self {
        Texture {
            image,
            memory,
            view,
            sampler,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            if let Some(sampler) = self.sampler.take() {
                device.destroy_sampler(sampler, None);
            }
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}
