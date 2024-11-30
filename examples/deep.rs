// Usage: cargo run --example deep

doko::doko!("examples/deepdir/deepsub", "print", ());

fn main() {
    doko_print("a")();
    doko_print("b")();
}
