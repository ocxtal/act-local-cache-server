use glob::glob;
use std::ops::Range;
use std::path::PathBuf;

pub fn parse_range(input: &str) -> Range<usize> {
    // parse "bytes 8388608-10485759/10485760" form

    // first split header
    let body = input.strip_prefix("bytes ").unwrap();

    let (range, _) = body.split_once('/').unwrap();
    let (start, end) = range.split_once('-').unwrap();

    let start = start.parse::<usize>().unwrap();
    let end = end.parse::<usize>().unwrap();

    start..end
}

pub fn glob_in(dir: &str, pattern: &str) -> Option<Vec<PathBuf>> {
    let paths = glob(&format!("{dir}/{pattern}")).ok()?;

    let mut array = Vec::new();
    for path in paths {
        array.push(path.ok()?.strip_prefix(dir).ok()?.to_path_buf());
    }
    Some(array)
}
