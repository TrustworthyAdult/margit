use std::env;
use std::path::PathBuf;
use std::process::Command;

fn resolve_git() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("MARGIT_GIT").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    let path_var = env::var_os("PATH").ok_or_else(|| "PATH is not set".to_string())?;

    for dir in env::split_paths(&path_var) {
        let candidate = dir.join("git");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err("could not locate `git` on PATH".to_string())
}

pub fn passthrough(args: Vec<String>) -> ! {
    let git = match resolve_git() {
        Ok(path) => path,
        Err(msg) => {
            eprintln!("margit: {msg}");
            std::process::exit(1);
        }
    };

    let result = Command::new(git).args(args).status();

    match result {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("margit: failed to run git: {err}");
            std::process::exit(1);
        }
    }
}
