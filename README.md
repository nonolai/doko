# `doko`

Run methods in submodules by name

## Overview

`doko` lets you include and run a method from a known submodule without importing and creating the
map from name to method yourself. Behind the scenes, the `doko::doko!` macro:

*   Finds all Rust files in the specified directory,
*   Includes them with `mod`
*   Creates function `doko_<function>` that takes a module name and returns
    `<module name>::<function>`. You can then call this function with whatever arguments you need.

Created to improve project layout for things like [Project Euler](https://projecteuler.net/) and
[Advent of Code](https://adventofcode.com/).

## Usage

### Project Layout

```rust
project/
 └──── src/
        ├──── main.rs
        └──── submod/
               ├───── a.rs
               └───── b.rs
```

### `project/src/submod/a.rs`

```rust
pub fn greeting(name: &str) -> u32 {
        println!("Hello, {}, from a", name);
        4
}
```

### `project/src/submod/b.rs`

```rust
pub fn greeting(name: &str) -> u32 {
        println!("Hello, {}, from b", name);
        5
}
```

### `project/src/main.rs`

```rust
doko::doko!("src/submod", "greeting", (&str) -> u32);

pub fn main() {
        let name = "username";
        assert_eq!(4, doko_greeting("a")(name));
        assert_eq!(5, doko_greeting("b")(name));
}
```

## TODO

*   Provide a cleaner API than generated functions (i.e.
    `let registry = doko::doko!(...); registry.run(<module>);`)
