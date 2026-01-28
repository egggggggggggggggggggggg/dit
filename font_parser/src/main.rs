use font_parser::TtfFont;

fn main() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    println!("Head: {:?}", font.head);
    println!("Maxp: {:?}", font.maxp);
    println!("Hhea: {:?}", font.hhea);
    // println!("Hmtx: {:?}", font.hmtx);
}
