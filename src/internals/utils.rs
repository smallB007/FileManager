use std::{fs, io, path};

pub fn read_directory(a_dir: &path::PathBuf) -> io::Result<Vec<path::PathBuf>> {
    let mut entries = fs::read_dir(a_dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.

    Ok(entries)
}
