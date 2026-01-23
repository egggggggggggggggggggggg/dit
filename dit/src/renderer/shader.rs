use std::io::Cursor;
use std::path::Path;

pub fn load<P: AsRef<Path>>(path: P) -> Cursor<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;

    let mut buf = Vec::new();
    let fullpath = Path::new("assets").join(path);
    let mut file = File::open(fullpath).unwrap();
    file.read_to_end(&mut buf).unwrap();
    Cursor::new(buf)
}
use ash::{Device, vk};
fn read_shader_from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<u32> {
    let mut cursor = load(path);
    ash::util::read_spv(&mut cursor).unwrap()
}

fn create_shader_module(device: &Device, code: &[u32]) -> vk::ShaderModule {
    let create_info = vk::ShaderModuleCreateInfo::default().code(code);
    unsafe { device.create_shader_module(&create_info, None).unwrap() }
}
