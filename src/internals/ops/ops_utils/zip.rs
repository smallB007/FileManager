use std::{io::prelude::*, path::PathBuf};

use zip::{result::ZipError, write::FileOptions};

use crate::internals::file_explorer_utils::get_file_content;

pub fn zip_file(input_file: &str, output_file: &str) -> zip::result::ZipResult<()> {
    let path = std::path::Path::new(output_file);
    let file_zipped = std::fs::File::create(&path)?;

    let mut zip = zip::ZipWriter::new(file_zipped);

    //zip.add_directory("test/", Default::default())?;

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    //.unix_permissions(0o755);
    let file_name = PathBuf::from(output_file)
        .file_name()
        .ok_or(ZipError::FileNotFound)?
        .to_str()
        .ok_or(ZipError::FileNotFound)?
        .to_string();

    zip.start_file(file_name, options)?;
    let buf = get_file_content(input_file)?;
    zip.write_all(&buf)?;
    zip.finish()?;

    Ok(())
}
