use dit::atlas::entry;
use std::time::Instant;
fn main() {
    let start = Instant::now();
    entry();
    println!("{:?}", start.elapsed());
}
