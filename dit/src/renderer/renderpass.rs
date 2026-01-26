use crate::renderer::{
    buffer::DynamicBuffer, swapchain::SwapchainProperties, texture::Texture,
    vkapp::MAX_FRAMES_IN_FLIGHT,
};
use ash::{Device, vk};
pub fn create_render_pass(
    device: &Device,
    swapchain_properties: SwapchainProperties,
    msaa_samples: vk::SampleCountFlags,
) -> vk::RenderPass {
    let color_attachment_desc = vk::AttachmentDescription::default()
        .format(swapchain_properties.format.format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let resolve_attachment_desc = vk::AttachmentDescription::default()
        .format(swapchain_properties.format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let attachment_descs = [color_attachment_desc, resolve_attachment_desc];
    let color_attachment_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_attachment_refs = [color_attachment_ref];
    let resolve_attachment_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let resolve_attachment_refs = [resolve_attachment_ref];

    let subpass_desc = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)
        .resolve_attachments(&resolve_attachment_refs);
    let subpass_descs = [subpass_desc];

    let subpass_dep = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        );
    let subpass_deps = [subpass_dep];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachment_descs)
        .subpasses(&subpass_descs)
        .dependencies(&subpass_deps);

    unsafe { device.create_render_pass(&render_pass_info, None).unwrap() }
}
pub fn create_framebuffers(
    device: &Device,
    image_views: &[vk::ImageView],
    color_texture: Texture,
    render_pass: vk::RenderPass,
    swapchain_properties: SwapchainProperties,
) -> Vec<vk::Framebuffer> {
    image_views
        .iter()
        .map(|view| [color_texture.view, *view])
        .map(|attachments| {
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_properties.extent.width)
                .height(swapchain_properties.extent.height)
                .layers(1);
            unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() }
        })
        .collect::<Vec<_>>()
}
#[derive(Clone, Copy)]
pub struct SyncObjects {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub fence: vk::Fence,
}

impl SyncObjects {
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.fence, None);
        }
    }
}
pub fn create_sync_objects(device: &Device) -> InFlightFrames {
    let mut sync_objects_vec = Vec::new();
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        let image_available_semaphore = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&semaphore_info, None).unwrap() }
        };

        let render_finished_semaphore = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe { device.create_semaphore(&semaphore_info, None).unwrap() }
        };

        let in_flight_fence = {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            unsafe { device.create_fence(&fence_info, None).unwrap() }
        };

        let sync_objects = SyncObjects {
            image_available_semaphore,
            render_finished_semaphore,
            fence: in_flight_fence,
        };
        sync_objects_vec.push(sync_objects)
    }

    InFlightFrames::new(sync_objects_vec)
}
pub struct InFlightFrames {
    sync_objects: Vec<SyncObjects>,
    current_frame: usize,
}

impl InFlightFrames {
    pub fn new(sync_objects: Vec<SyncObjects>) -> Self {
        Self {
            sync_objects,
            current_frame: 0,
        }
    }
    pub fn destroy(&self, device: &Device) {
        self.sync_objects.iter().for_each(|o| o.destroy(device));
    }
    //Returns the next resource
    //Dynamic vertex buffer + Sync Objects for the
    fn unitered_next() {}
}

impl Iterator for InFlightFrames {
    type Item = SyncObjects;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.sync_objects[self.current_frame];

        self.current_frame = (self.current_frame + 1) % self.sync_objects.len();

        Some(next)
    }
}
// #[derive(Clone, Copy)]
// struct FrameResources {
//     sync_object: SyncObjects,
//     dynamic_vertex_buffer: DynamicBuffer,
// }
// impl FrameResources {
//     fn destroy(&self, device: &Device) {
//         self.sync_object.destroy(device);
//         self.dynamic_vertex_buffer.destroy();
//     }
// }
// struct FramesInFlight {
//     frame_resources: Vec<FrameResources>,
//     current_frame: usize,
// }
// impl FramesInFlight {
//     fn new(frame_resources: Vec<FrameResources>) -> Self {
//         Self {
//             frame_resources,
//             current_frame: 0,
//         }
//     }
//     fn destroy(&self, device: &Device) {
//         self.frame_resources.iter().for_each(|o| o.destroy(device));
//     }
// }
// impl Iterator for FramesInFlight {
//     type Item = FrameResources;
//     fn next(&mut self) -> Option<Self::Item> {
//         let frame_resources = self.frame_resources[self.current_frame];
//         self.current_frame = (self.current_frame + 1) & self.frame_resources.len();
//         Some(frame_resources)
//     }
// }
