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
#[derive(Clone, Copy, Debug)]
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
pub struct InstanceData {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv: [f32; 4],
    pub fg: [u8; 4],
    pub bg: [u8; 4],
}
impl InstanceData {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(1)
            .stride(size_of::<InstanceData>() as _)
            .input_rate(vk::VertexInputRate::INSTANCE)
    }
    fn attribute_description() -> [vk::VertexInputAttributeDescription; 5] {
        let pos_desc = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(InstanceData, pos) as _);
        let size_desc = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(3)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(InstanceData, size) as _);
        let uv_desc = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(4)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(InstanceData, uv) as _);
        let fg_desc = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(5)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(InstanceData, fg) as _);
        let bg_desc = vk::VertexInputAttributeDescription::default()
            .binding(1)
            .location(6)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(InstanceData, bg) as _);
        [pos_desc, uv_desc, size_desc, fg_desc, bg_desc]
    }
}
