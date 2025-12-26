use ash::{
    Device, Entry, khr,
    vk::{self, ApplicationInfo},
};
pub fn entry() {}
pub fn createInstace() {
    let entry = unsafe { Entry::load() }.unwrap();
    let app_info = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 0, 0));
}
