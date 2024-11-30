// Usage: cargo run --example demo

doko::doko!("examples/utilities", "run", () -> u32);

fn main() {
    assert_eq!(1, doko_run("first")());
    assert_eq!(2, doko_run("second")());
}
