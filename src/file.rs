use crate::utils::glob_in;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::Path;

pub fn save_file(path: &str, is_gzip: bool, start: usize, input: &[u8]) {
    // workaround for gzipped stream
    let gz = if is_gzip { ".gzippedStream" } else { "" };

    // format chunk prefix that can be safely sorted into the original chunk order
    // (this assumes total bytes being less than 1TB)
    let path = format!("{path}{gz}.uploadTemporary.{start:012}");

    let (dir, _) = path.rsplit_once('/').unwrap();
    std::fs::create_dir_all(dir).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    file.write_all(input).unwrap();
}

pub fn finalize_files(dir: &str, pattern: &str) -> usize {
    let paths = glob_in(&dir, &format!("{pattern}.uploadTemporary.*")).unwrap();

    // group by basename
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for path in &paths {
        let path = path.to_str().unwrap();

        let (basename, _) = path.split_once(".uploadTemporary.").unwrap();
        let basename = basename.to_string();

        if let Some(slot) = map.get_mut(&basename) {
            slot.push(path.to_string());
        } else {
            map.insert(basename, vec![path.to_string()]);
        }
    }

    // concat all
    let mut acc = 0;
    let mut buf = Vec::new();
    for (dst, srcs) in &mut map {
        srcs.sort();

        let dst = format!("{dir}/{dst}");
        let mut dst = std::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dst)
            .unwrap();

        for src in srcs {
            buf.clear();

            let src = format!("{dir}/{src}");
            {
                let mut src = std::fs::File::open(&src).unwrap();
                acc += src.read_to_end(&mut buf).unwrap();
            }

            dst.write_all(&buf).unwrap();
            std::fs::remove_file(&src).unwrap();
        }
        dst.sync_all().unwrap();
    }

    acc
}

pub fn list_all_files(dir: &str) -> Vec<String> {
    let paths = glob_in(dir, "**/*").unwrap();

    let mut array = Vec::new();
    for path in &paths {
        let path = path.to_str().unwrap();

        // workaround for gzipped stream; remove if the file has the .gzippedStream prefix
        let path = path.strip_suffix(".gzippedStream").unwrap_or(path);

        if !Path::new(&format!("{dir}/{path}")).is_dir() {
            array.push(path.to_string());
        }
    }
    array
}

pub fn dump_file(path: &str, range: Option<Range<usize>>) -> (bool, Vec<u8>) {
    // workaround for gzipped stream
    let gzipped = format!("{path}.gzippedStream");
    let (is_gzip, path) = if Path::new(&gzipped).exists() {
        (true, gzipped.as_str())
    } else {
        (false, path)
    };

    let mut file = std::fs::File::open(path).unwrap();

    // slice the specified range
    if let Some(range) = range {
        file.seek(SeekFrom::Start(range.start as u64)).unwrap();

        let mut buf = Vec::with_capacity(range.len());
        unsafe { buf.set_len(range.len()) };

        let len = file.read(&mut buf).unwrap();
        unsafe { buf.set_len(len) };

        (is_gzip, buf)
    } else {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        (is_gzip, buf)
    }
}
