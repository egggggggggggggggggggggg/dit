pub mod buffer;
mod command;
mod context;
mod debug;
mod device;
mod pipeline;
mod queue;
mod renderpass;
mod resources;
pub mod shader;
mod swapchain;
mod texture;
mod utils;
pub mod vkapp;

// mod vkcore;

use buffer::*;
use command::*;
use context::*;
use debug::*;
use device::*;
use pipeline::*;
use renderpass::*;
use resources::*;
use shader::*;
use swapchain::*;
use texture::*;
use utils::*;

#[derive(Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
impl Mesh {
    pub fn reset(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}
