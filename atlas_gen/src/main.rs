use atlas_gen::{
    allocator::ShelfAllocator,
    atlas::Atlas,
    contour_combiner::{OverlappingContourCombiner, SimpleContourCombiner},
    edge_selectors::{MultiDistanceSelector, PerpendicularDistanceSelector},
};
use font_parser::TtfFont;
use image::Rgb;

fn main() {
    entry();
}
fn entry() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(512, 512);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator, 4, false);
    for ch in '!'..'~' {
        let gid = font.lookup(ch as u32).unwrap();
        let shape = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let header = glyph.get_header();
        let mut scc: SimpleContourCombiner<MultiDistanceSelector> =
            SimpleContourCombiner::new(shape);

        println!("test: {:?}", header);
    }
}
