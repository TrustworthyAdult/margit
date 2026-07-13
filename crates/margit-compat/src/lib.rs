use std::env::{self, current_exe};
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
enum ResolveError {
    PathNotSet,
    NotFound,
    SelfReference,
    SelfUnknown(std::io::Error),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::PathNotSet => write!(f, "PATH is not set"),
            ResolveError::NotFound => write!(f, "could not locate `git` on PATH"),
            ResolveError::SelfReference => {
                write!(
                    f,
                    "MARGIT_GIT points at margit itself; that would loop forever"
                )
            }
            ResolveError::SelfUnknown(e) => {
                write!(f, "could not locate margit's own executable: {e}")
            }
        }
    }
}

impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ResolveError::SelfUnknown(e) => Some(e),
            _ => None,
        }
    }
}

fn resolve_git() -> Result<PathBuf, ResolveError> {
    let me = current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(ResolveError::SelfUnknown)?;
    resolve_git_from(env::var_os("MARGIT_GIT"), env::var_os("PATH"), &me)
}

fn resolve_git_from(
    override_var: Option<OsString>,
    path_var: Option<OsString>,
    me: &Path,
) -> Result<PathBuf, ResolveError> {
    if let Some(value) = override_var.filter(|v| !v.is_empty()) {
        let candidate = PathBuf::from(value);
        if is_margit(&candidate, me) {
            return Err(ResolveError::SelfReference);
        }
        return Ok(candidate);
    }

    let path_var = path_var.ok_or(ResolveError::PathNotSet)?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join("git");
        if candidate.is_file() && !is_margit(&candidate, me) {
            return Ok(candidate);
        }
    }
    Err(ResolveError::NotFound)
}

fn is_margit(path: &Path, margit: &Path) -> bool {
    let Ok(canon) = path.canonicalize() else {
        return false;
    };
    canon == margit
}

pub fn passthrough(args: Vec<String>) -> ! {
    let git = match resolve_git() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("margit: {err}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_is_an_error() {
        let me = PathBuf::from("/nonexistent/margit");
        assert!(matches!(
            resolve_git_from(None, None, &me),
            Err(ResolveError::PathNotSet)
        ));
    }

    #[test]
    fn empty_override_is_ignored() {
        let me = PathBuf::from("/nonexistent/margit");
        // Empty MARGIT_GIT must fall through; with no PATH that's PathNotSet.
        assert!(matches!(
            resolve_git_from(Some(OsString::new()), None, &me),
            Err(ResolveError::PathNotSet)
        ));
    }

    #[test]
    fn override_is_returned_verbatim() {
        let me = PathBuf::from("/nonexistent/margit");
        // A non-existent override can't be canonicalized, so it isn't "us".
        let result = resolve_git_from(Some(OsString::from("/nonexistent/git")), None, &me);
        assert_eq!(result.unwrap(), PathBuf::from("/nonexistent/git"));
    }

    #[test]
    fn returns_git_found_on_path() {
        let dir = env::temp_dir().join(format!("margit-find-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let git = dir.join("git");
        std::fs::write(&git, b"").unwrap();

        let me = PathBuf::from("/nonexistent/margit");
        let result = resolve_git_from(None, Some(dir.clone().into_os_string()), &me);

        assert_eq!(result.unwrap(), git);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn skips_a_git_that_is_actually_us() {
        use std::os::unix::fs::symlink;

        let dir = env::temp_dir().join(format!("margit-self-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        // A stand-in for the margit binary, plus its canonical path.
        let me_bin = dir.join("margit-bin");
        std::fs::write(&me_bin, b"").unwrap();
        let me = me_bin.canonicalize().unwrap();

        // `git` on PATH is a symlink back to us → must be skipped.
        symlink(&me, dir.join("git")).unwrap();

        let result = resolve_git_from(None, Some(dir.clone().into_os_string()), &me);

        // The only candidate was ourselves, so nothing real was found.
        assert!(matches!(result, Err(ResolveError::NotFound)));
        std::fs::remove_dir_all(&dir).ok();
    }
}
