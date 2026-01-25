use crate::renderer::utils::load;
use ash::{Device, vk};
use std::mem::offset_of;

pub fn read_shader_from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<u32> {
    let mut cursor = load(path);
    ash::util::read_spv(&mut cursor).unwrap()
}

pub fn create_shader_module(device: &Device, code: &[u32]) -> vk::ShaderModule {
    let create_info = vk::ShaderModuleCreateInfo::default().code(code);
    unsafe { device.create_shader_module(&create_info, None).unwrap() }
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}
impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as _)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let position_desc = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, pos) as _);
        let uv_desc = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, uv) as _);
        [position_desc, uv_desc]
    }
}
