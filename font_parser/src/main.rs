use font_parser::TtfFont;

fn main() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let gid = font.lookup('a' as u32).unwrap();
    //maintain a pen that holds the value of teh advanced width
    //eg after a lookup its advance_width valuen and add to the pen
    //use the pen for the base of the next character
    let glyf = font.parse_gid(gid as u16).unwrap().unwrap().get_header();
    let units_per_em = font.head.units_per_em;
    let cell_advance = font.hmtx.metric_for_glyph(gid as u16).advance_width;
    let cell_ascent = font.hhea.ascent;
    let cell_descent = -font.hhea.descent;
    let cell_height = cell_ascent - cell_descent + font.hhea.line_gap;
    let font_size_px = 16;
    let scale = font_size_px as f32 / units_per_em as f32;
    let cell_width_px = cell_advance as f32 * scale;
    let cell_height_px = cell_height as f32 * scale;
    let baseline_offset_px = cell_ascent as f32 * scale;
    //example usage
    let col = 1;
    let row = 1;
    let x_cell = col as f32 * cell_width_px;
    let y_cell = row as f32 * cell_height_px;
    let baseline_x = x_cell;
    let baseline_y = y_cell + baseline_offset_px;
    let screen_x = baseline_x + (glyf.x * scale);
    let screen_y = baseline_y - (glyph_y * scale);

    println!("baseline_offset_px: {}", baseline_offset_px);
}
