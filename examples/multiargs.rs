// Usage: cargo run --example multiargs

doko::doko!("examples/multiargsdir", "cmp", (u32, u32) -> bool);

fn main() {
    println!("{} < {} ? {}", 5, 13, doko_cmp("lt")(5, 13));
    println!("{} > {} ? {}", 5, 13, doko_cmp("gt")(5, 13));
}
