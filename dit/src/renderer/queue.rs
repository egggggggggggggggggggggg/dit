use ash::{Instance, khr::surface, vk};
#[derive(Clone, Copy)]
pub struct QueueFamiliesIndices {
    pub graphics_index: u32,
    pub present_index: u32,
    pub transfer_index: u32,
}
impl QueueFamiliesIndices {
    fn new(graphics: u32, present: u32, transfer: u32) -> Self {
        Self {
            graphics_index: graphics,
            present_index: present,
            transfer_index: transfer,
        }
    }
}
pub fn find_queue_families(
    instance: &Instance,
    surface: &surface::Instance,
    surface_khr: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    let mut graphics = None;
    let mut present = None;
    let mut transfer = None;
    let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
    for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate() {
        let index = index as u32;

        if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
            graphics = Some(index);
        }

        let present_support = unsafe {
            surface
                .get_physical_device_surface_support(device, index, surface_khr)
                .unwrap()
        };
        if present_support && present.is_none() {
            present = Some(index);
        }

        // Prefer a dedicated transfer queue
        if family.queue_flags.contains(vk::QueueFlags::TRANSFER)
            && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && !family.queue_flags.contains(vk::QueueFlags::COMPUTE)
            && transfer.is_none()
        {
            transfer = Some(index);
        }
    }

    // Fallback: use graphics queue for transfer if needed
    if transfer.is_none() {
        transfer = graphics;
    }
    (graphics, present, transfer)
}
