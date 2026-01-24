use std::fs;
use std::path::Path;

fn list_fonts() -> std::io::Result<()> {
    // #[cfg(target_os = "windows")]
    // let font_dir = Path::new("/usr/share/fonts");
    //windows integration might be a bit annoying
    #[cfg(target_os = "linux")]
    let font_dir = Path::new("/usr/share/fonts");
    for entry in fs::read_dir(font_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            println!("Dir: {:?}", path);
        } else {
            println!("File: {:?}", path);
        }
    }

    Ok(())
}
#[derive(Debug, Clone)]
struct Folder {
    name: &'static str,
    nodes: Vec<Node>,
}
#[derive(Debug, Clone)]
enum Node {
    Directory(&'static str),
    File(Folder),
}
pub struct Root {
    pub files: Vec<Node>,
}
impl Root {
    fn new() {}
    fn find_file(&mut self, file_name: &str) {
        let mut root = &self.files.clone();
    }
}

fn search_fonts(font_names: Vec<String>, desired_font_name: String) {}
