use glob::glob;
use std::path::PathBuf;

pub fn glob_in(dir: &str, pattern: &str) -> Option<Vec<PathBuf>> {
    let paths = glob(&format!("{dir}/{pattern}")).ok()?;

    let mut array = Vec::new();
    for path in paths {
        array.push(path.ok()?.strip_prefix(dir).ok()?.to_path_buf());
    }
    Some(array)
}
