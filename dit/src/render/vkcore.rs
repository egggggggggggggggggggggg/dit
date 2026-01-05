use crate::render::debug;
use ash::{Entry, khr::surface, vk};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};
struct VkApp {}
impl VkApp {
    fn new(window: &Window) {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = debug::create_instance(&entry, window);
        let surface = surface::Instance::new(&entry, &instance);
        let surface_khr = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap();
        };
    }
}
