use std::io::Cursor;
use std::path::Path;
pub fn load<P: AsRef<Path>>(path: P) -> Cursor<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;

    let mut buf = Vec::new();
    let fullpath = Path::new("assets").join(path);
    println!("{:?}", fullpath);
    let mut file = File::open(fullpath).unwrap();
    file.read_to_end(&mut buf).unwrap();
    Cursor::new(buf)
}
use ash::vk;
pub fn find_memory_type(
    requirements: vk::MemoryRequirements,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    required_properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    panic!("Failed to find suitable memory type.")
}
