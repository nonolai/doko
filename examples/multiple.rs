// Usage: cargo run --example multiple

doko::doko!("examples/multipledir", "method_1", () -> String);
doko::doko_skip_mods!("examples/multipledir", "method_2", () -> String);

fn main() {
    assert_eq!("Mod: A, Method: 1", doko_method_1("a")());
    assert_eq!("Mod: A, Method: 2", doko_method_2("a")());
    assert_eq!("Mod: B, Method: 1", doko_method_1("b")());
    assert_eq!("Mod: B, Method: 2", doko_method_2("b")());
}
