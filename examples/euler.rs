// Usage: cargo run --example euler <problem numbers>

use std::env;

doko::doko!("examples/solutions", "solve", () -> String);

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        println!("Usage: cargo run --example euler <problem numbers>");
        return;
    }

    for argument in args {
        let problem_number = format!("{:03}", argument.parse::<u32>().unwrap());
        let module = format!("s{}", problem_number);
        let solution = doko_solve(&module)();
        println!("Solution [{}]: {}", problem_number, solution);
    }
}
