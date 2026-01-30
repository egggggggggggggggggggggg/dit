use math::lalg::BinaryVector;

fn seed_extract2(seed: &mut u64) -> i32 {
    let v = (*seed & 1) as i32;
    *seed >>= 1;
    v
}

fn seed_extract3(seed: &mut u64) -> i32 {
    let v = (*seed % 3) as i32;
    *seed /= 3;
    v
}

fn init_color(seed: &mut u64) -> BinaryVector {
    const COLORS: [BinaryVector; 3] = [
        BinaryVector::CYAN,
        BinaryVector::MAGENTA,
        BinaryVector::YELLOW,
    ];
    COLORS[seed_extract3(seed) as usize]
}
fn switch_color(color: BinaryVector, seed: &mut u64) {}

fn edge_coloring_simple() {}
