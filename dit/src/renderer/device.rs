use std::ffi::CStr;

use crate::renderer::{
    queue::{QueueFamiliesIndices, find_queue_families},
    swapchain::SwapchainSupportDetails,
};
use ash::{Device, Instance, khr::surface, vk};
pub fn pick_physical_device(
    instance: &Instance,
    surface: &surface::Instance,
    surface_khr: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, QueueFamiliesIndices) {
    println!("printing instance info");
    let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    let device = devices
        .into_iter()
        .find(|device| is_device_suitable(instance, surface, surface_khr, *device))
        .expect("No suitable physical device.");
    let (graphics, present, transfer) = find_queue_families(instance, surface, surface_khr, device);
    let queue_families_indices = QueueFamiliesIndices {
        graphics_index: graphics.unwrap(),
        present_index: present.unwrap(),
        transfer_index: transfer.unwrap(),
    };
    let props = unsafe { instance.get_physical_device_properties(device) };
    println!("properties of device: {:?}", props.limits);
    (device, queue_families_indices)
}

fn is_device_suitable(
    instance: &Instance,
    surface: &surface::Instance,
    surface_khr: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> bool {
    let (graphics, present, transfer) = find_queue_families(instance, surface, surface_khr, device);
    let extension_support = check_device_extension_support(instance, device);
    let is_swapchain_adequate = {
        let details = SwapchainSupportDetails::new(device, surface, surface_khr);
        !details.formats.is_empty() && !details.present_modes.is_empty()
    };
    let features = unsafe { instance.get_physical_device_features(device) };
    graphics.is_some()
        && present.is_some()
        && extension_support
        && is_swapchain_adequate
        && features.sampler_anisotropy == vk::TRUE
}
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn get_required_device_extensions() -> [&'static CStr; 1] {
    [ash::khr::swapchain::NAME]
}
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn get_required_device_extensions() -> [&'static CStr; 2] {
    [
        ash::khr::swapchain::NAME,
        ash::khr::portability_subset::NAME,
    ]
}
fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    let required_extentions = get_required_device_extensions();

    let extension_props = unsafe {
        instance
            .enumerate_device_extension_properties(device)
            .unwrap()
    };

    for required in required_extentions.iter() {
        let found = extension_props.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            required == &name
        });

        if !found {
            return false;
        }
    }

    true
}
pub fn create_logical_device_with_graphics_queue(
    instance: &Instance,
    device: vk::PhysicalDevice,
    queue_families_indices: QueueFamiliesIndices,
) -> (Device, vk::Queue, vk::Queue) {
    let graphics_family_index = queue_families_indices.graphics_index;
    let present_family_index = queue_families_indices.present_index;
    let queue_priorities = [1.0f32];
    let queue_create_infos = {
        let mut indices = vec![graphics_family_index, present_family_index];
        indices.dedup();
        indices
            .iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*index)
                    .queue_priorities(&queue_priorities)
            })
            .collect::<Vec<_>>()
    };
    let device_extensions = get_required_device_extensions();
    let device_extensions_ptrs = device_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    let device_features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);
    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extensions_ptrs)
        .enabled_features(&device_features);
    let device = unsafe {
        instance
            .create_device(device, &device_create_info, None)
            .expect("Failed to create logical device.")
    };
    let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
    let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };
    (device, graphics_queue, present_queue)
}
