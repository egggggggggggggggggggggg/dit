pub trait AtlasAllocator {
    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)>;
    fn dimensions(&self) -> (u32, u32);
}
pub struct ShelfAllocator {
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
}
struct Shelf {
    y: u32,
    height: u32,
    x_cursor: u32,
}
impl ShelfAllocator {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::new(),
        }
    }
}
impl AtlasAllocator for ShelfAllocator {
    fn allocate(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        for shelf in &mut self.shelves {
            if h <= shelf.height && shelf.x_cursor + w <= self.width {
                let x = shelf.x_cursor;
                let y = shelf.y;
                shelf.x_cursor += w;
                return Some((x, y));
            }
        }
        let y = self.shelves.last().map(|s| s.y + s.height).unwrap_or(0);
        if y + h > self.height {
            return None;
        }
        self.shelves.push(Shelf {
            y,
            height: h,
            x_cursor: w,
        });
        Some((0, y))
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
pub struct SkylineAllocator {
    width: u32,
    height: u32,
}
impl SkylineAllocator {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
