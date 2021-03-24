use std::{fs::File, io::prelude::*};

use zip::write::FileOptions;

pub fn zip_file(input_file: &str, output_file: &str) -> zip::result::ZipResult<()> {
    let path = std::path::Path::new(output_file);
    let file = std::fs::File::create(&path).unwrap();

    let mut zip = zip::ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file(output_file, options)?;

    let mut f = File::open(input_file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    zip.write_all(&buf)?; //todo bufferred
    zip.finish()?;

    Ok(())
}
