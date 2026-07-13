use std::process::Command;
pub fn passthrough(args: Vec<String>) -> ! {
    let result = Command::new("git").args(args).status();

    match result {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("margit: failed to run git: {err}");
            std::process::exit(1);
        }
    }
}
