use crate::renderer::*;
use crate::renderer::{queue::QueueFamiliesIndices, texture::Texture};
use ash::{
    Device, Entry, Instance,
    khr::{surface, swapchain as khr_swapchain},
    vk,
};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};
const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct VkApplication {
    pub resize_dimensions: [u32; 2],
    pub is_left_clicked: bool,
    pub cursor_position: [i32; 2],
    pub cursor_delta: Option<[i32; 2]>,
    pub wheel_delta: Option<f32>,
    pub dirty_swapchain: bool,
    pub vk_context: VkContext,
    pub queue_families_indices: QueueFamiliesIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain: khr_swapchain::Device,
    pub swapchain_khr: vk::SwapchainKHR,
    pub swapchain_properties: SwapchainProperties,
    pub images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub swapchain_framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub transient_command_pool: vk::CommandPool,
    pub msaa_samples: vk::SampleCountFlags,
    pub color_texture: texture::Texture,
    pub texture: texture::Texture,
    pub model_index_count: usize,
    pub vertex_buffer: DynamicBuffer,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffer_memories: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub in_flight_frames: InFlightFrames,
}
//Buffer Alloc Size is derived from the initial screen size
//Allows for pre allocating space to the vertex buffer
//Prevents excession reallocation later on
impl VkApplication {
    pub fn new(window: &Window, buffer_alloc_size: u64) -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = create_instance(&entry, window);
        let surface = surface::Instance::new(&entry, &instance);
        let surface_khr = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        };
        println!("working now");
        let debug_report_callback = setup_debug_messenger(&entry, &instance);
        let (physical_device, queue_families_indices) =
            pick_physical_device(&instance, &surface, surface_khr);

        let (device, graphics_queue, present_queue) = create_logical_device_with_graphics_queue(
            &instance,
            physical_device,
            queue_families_indices,
        );
        let vk_context = VkContext::new(
            entry,
            instance,
            debug_report_callback,
            surface,
            surface_khr,
            physical_device,
            device,
        );
        let (swapchain, swapchain_khr, properties, images) =
            create_swapchain_and_images(&vk_context, queue_families_indices, [WIDTH, HEIGHT]);
        let swapchain_image_views =
            create_swapchain_image_views(vk_context.device(), &images, properties);
        let msaa_samples = vk_context.get_max_usable_sample_count();
        let render_pass = create_render_pass(vk_context.device(), properties, msaa_samples);
        let descriptor_set_layout = create_descriptor_set_layout(vk_context.device());
        let (pipeline, layout) = create_pipeline(
            vk_context.device(),
            properties,
            msaa_samples,
            render_pass,
            descriptor_set_layout,
        );
        let command_pool = create_command_pool(
            vk_context.device(),
            queue_families_indices,
            vk::CommandPoolCreateFlags::empty(),
        );
        let transient_command_pool = create_command_pool(
            vk_context.device(),
            queue_families_indices,
            vk::CommandPoolCreateFlags::TRANSIENT,
        );
        let color_texture = create_color_texture(
            &vk_context,
            command_pool,
            graphics_queue,
            properties,
            msaa_samples,
        );
        let swapchain_framebuffers = create_framebuffers(
            vk_context.device(),
            &swapchain_image_views,
            color_texture,
            render_pass,
            properties,
        );

        let texture = create_texture_image(&vk_context, command_pool, graphics_queue);
        let (vertices, indices) = Self::load_model();

        let mut dynamic_vertex_buffer = DynamicBuffer::new(
            (byte_size(&vertices) * 4) as vk::DeviceSize,
            &vk_context,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );
        dynamic_vertex_buffer.full_copy::<u32, Vertex>(
            &vk_context,
            command_pool,
            graphics_queue,
            &vertices,
        );
        let device_buffer = dynamic_vertex_buffer.device;

        let (vertex_buffer, vertex_buffer_memory) = (device_buffer.buffer, device_buffer.memory);
        // let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
        //     &vk_context,
        //     transient_command_pool,
        //     graphics_queue,
        //     &vertices,
        // );
        let (index_buffer, index_buffer_memory) = create_index_buffer(
            &vk_context,
            transient_command_pool,
            graphics_queue,
            &indices,
        );

        let (uniform_buffers, uniform_buffer_memories) =
            create_uniform_buffers(&vk_context, images.len());

        let descriptor_pool = create_descriptor_pool(vk_context.device(), images.len() as _);
        let descriptor_sets = create_descriptor_sets(
            vk_context.device(),
            descriptor_pool,
            descriptor_set_layout,
            &uniform_buffers,
            texture,
        );
        let command_buffers = create_and_register_command_buffers(
            vk_context.device(),
            command_pool,
            &swapchain_framebuffers,
            render_pass,
            properties,
            vertex_buffer,
            index_buffer,
            indices.len(),
            layout,
            &descriptor_sets,
            pipeline,
        );
        let in_flight_frames = create_sync_objects(vk_context.device());

        Self {
            resize_dimensions: [WIDTH, HEIGHT],
            is_left_clicked: false,
            cursor_position: [0, 0],
            cursor_delta: None,
            wheel_delta: None,
            dirty_swapchain: false,
            vk_context,
            queue_families_indices,
            graphics_queue,
            present_queue,
            swapchain,
            swapchain_khr,
            swapchain_properties: properties,
            images,
            swapchain_image_views,
            render_pass,
            descriptor_set_layout,
            pipeline_layout: layout,
            pipeline,
            swapchain_framebuffers,
            command_pool,
            transient_command_pool,
            msaa_samples,
            color_texture,
            texture,
            model_index_count: indices.len(),
            vertex_buffer: dynamic_vertex_buffer,
            index_buffer,
            index_buffer_memory,
            uniform_buffers,
            uniform_buffer_memories,
            descriptor_pool,
            descriptor_sets,
            command_buffers,
            in_flight_frames,
        }
    }
    pub fn wait_gpu_idle(&self) {
        unsafe { self.vk_context.device().device_wait_idle().unwrap() };
    }
    fn load_model() -> (Vec<Vertex>, Vec<u32>) {
        let vertices = [
            Vertex {
                pos: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                pos: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
        ]
        .to_vec();
        let indices = [0u32, 1, 2, 2, 3, 0].to_vec();
        (vertices, indices)
    }
    pub fn draw_frame(&mut self) -> bool {
        let sync_objects = self.in_flight_frames.next().unwrap();
        let image_available_semaphore = sync_objects.image_available_semaphore;
        let render_finished_semaphore = sync_objects.render_finished_semaphore;
        let in_flight_fence = sync_objects.fence;
        let wait_fences = [in_flight_fence];
        unsafe {
            self.vk_context
                .device()
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .unwrap();
            self.vk_context.device().reset_fences(&wait_fences).unwrap();
        };
        let result = unsafe {
            self.swapchain.acquire_next_image(
                self.swapchain_khr,
                u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            )
        };
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return true;
            }
            Err(error) => panic!("Error while acquiring next image. Cause: {}", error),
        };
        let device = self.vk_context.device();
        let wait_semaphores = [image_available_semaphore];
        let signal_semaphores = [render_finished_semaphore];
        {
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.command_buffers[image_index as usize]];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            let submit_infos = [submit_info];
            unsafe {
                device
                    .queue_submit(self.graphics_queue, &submit_infos, in_flight_fence)
                    .unwrap()
            };
        }
        let swapchains = [self.swapchain_khr];
        let images_indices = [image_index];

        {
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&images_indices);
            // .results() null since we only have one swapchain
            let result = unsafe {
                self.swapchain
                    .queue_present(self.present_queue, &present_info)
            };
            match result {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return true;
                }
                Err(error) => panic!("Failed to present queue. Cause: {}", error),
                _ => {}
            }
        }
        false
    }
    pub fn test_dynamic_buffer(&mut self) {
        let vertices = vec![
            Vertex {
                pos: [-10.0, -3.0],
                uv: [-2.0, -2.0],
            },
            Vertex {
                pos: [-8.0, 12.0],
                uv: [-1.0, 3.0],
            },
            Vertex {
                pos: [6.0, 9.0],
                uv: [4.0, 2.0],
            },
            Vertex {
                pos: [12.0, -6.0],
                uv: [5.0, -1.0],
            },
        ];

        self.vertex_buffer.full_copy::<u32, _>(
            &self.vk_context,
            self.command_pool,
            self.graphics_queue,
            &vertices,
        );
        //acquire the buffer
    }
    fn cleanup_swapchain(&mut self) {
        let device = self.vk_context.device();
        unsafe {
            self.color_texture.destroy(device);
            self.swapchain_framebuffers
                .iter()
                .for_each(|f| device.destroy_framebuffer(*f, None));
            device.free_command_buffers(self.command_pool, &self.command_buffers);
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_render_pass(self.render_pass, None);
            self.swapchain_image_views
                .iter()
                .for_each(|v| device.destroy_image_view(*v, None));
            self.swapchain.destroy_swapchain(self.swapchain_khr, None);
        }
    }
    pub fn recreate_swapchain(&mut self) {
        self.wait_gpu_idle();

        self.cleanup_swapchain();

        let device = self.vk_context.device();

        let dimensions = self.resize_dimensions;
        let (swapchain, swapchain_khr, properties, images) =
            create_swapchain_and_images(&self.vk_context, self.queue_families_indices, dimensions);
        let swapchain_image_views = create_swapchain_image_views(device, &images, properties);

        let render_pass = create_render_pass(device, properties, self.msaa_samples);
        let (pipeline, layout) = create_pipeline(
            device,
            properties,
            self.msaa_samples,
            render_pass,
            self.descriptor_set_layout,
        );

        let color_texture = create_color_texture(
            &self.vk_context,
            self.command_pool,
            self.graphics_queue,
            properties,
            self.msaa_samples,
        );

        let swapchain_framebuffers = create_framebuffers(
            device,
            &swapchain_image_views,
            color_texture,
            render_pass,
            properties,
        );

        let command_buffers = create_and_register_command_buffers(
            device,
            self.command_pool,
            &swapchain_framebuffers,
            render_pass,
            properties,
            self.vertex_buffer.device.buffer,
            self.index_buffer,
            self.model_index_count,
            layout,
            &self.descriptor_sets,
            pipeline,
        );

        self.swapchain = swapchain;
        self.swapchain_khr = swapchain_khr;
        self.swapchain_properties = properties;
        self.images = images;
        self.swapchain_image_views = swapchain_image_views;
        self.render_pass = render_pass;
        self.pipeline = pipeline;
        self.pipeline_layout = layout;
        self.color_texture = color_texture;
        self.swapchain_framebuffers = swapchain_framebuffers;
        self.command_buffers = command_buffers;
    }
}
impl Drop for VkApplication {
    fn drop(&mut self) {
        self.cleanup_swapchain();

        let device = self.vk_context.device();
        self.in_flight_frames.destroy(device);
        unsafe {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.uniform_buffer_memories
                .iter()
                .for_each(|m| device.free_memory(*m, None));
            self.uniform_buffers
                .iter()
                .for_each(|b| device.destroy_buffer(*b, None));
            device.free_memory(self.index_buffer_memory, None);
            device.destroy_buffer(self.index_buffer, None);
            self.vertex_buffer.destroy(device);
            self.texture.destroy(device);
            device.destroy_command_pool(self.transient_command_pool, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
