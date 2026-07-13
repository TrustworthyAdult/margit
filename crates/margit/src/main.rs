use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--version") {
        println!("margit {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    margit_compat::passthrough(args);
}
