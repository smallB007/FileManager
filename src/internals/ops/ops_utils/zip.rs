use std::{
    io::prelude::*,
    path::{Path, PathBuf},
};

use zip::{result::ZipError, write::FileOptions};

use crate::internals::file_explorer_utils::get_file_content;

pub fn zip_file(input_file: &str, filename: &str) -> zip::result::ZipResult<()> {
    let output_file = PathBuf::from(input_file);
    let output_file = output_file.parent().unwrap();
    let output_file = output_file.join(String::from(filename) + ".zip");
    let path = std::path::Path::new(&output_file); //archive name
    let file = std::fs::File::create(&path).unwrap();

    let mut zip = zip::ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    let file_name = PathBuf::from(input_file)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    zip.start_file(file_name, options)?; //file name inside archive
    let buf = get_file_content(input_file)?;
    zip.write_all(&buf)?;

    zip.finish()?;
    Ok(())
}
