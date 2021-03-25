use std::{fs::File, io::prelude::*};

use zip::write::FileOptions;

use crate::internals::file_explorer_utils::get_file_content;

pub fn zip_file(input_file: &str, output_file: &str) -> zip::result::ZipResult<()> {
    let path = std::path::Path::new(output_file);
    let file = std::fs::File::create(&path).unwrap();

    let mut zip = zip::ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file(output_file, options)?;

    let mut buf = get_file_content(input_file)?;

    zip.write_all(&buf)?; //todo bufferred
    zip.finish()?;

    Ok(())
}
