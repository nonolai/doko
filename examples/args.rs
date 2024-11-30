// Usage: cargo run --example args

doko::doko!("examples/argssub", "check", (u32) -> bool);

fn main() {
    for i in 0..10 {
        println!(
            "{}: Odd? {:5} Even? {:5}",
            i,
            doko_check("is_odd")(i),
            doko_check("is_even")(i)
        );
    }
}
