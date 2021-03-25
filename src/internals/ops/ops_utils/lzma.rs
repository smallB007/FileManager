use std::fs::File;
use std::io::Write;

use lzma::{LzmaError, LzmaWriter};

use crate::internals::file_explorer_utils::get_file_content;

pub fn create_xz_archive(input_file: &str, output_file: &str) -> Result<(), LzmaError> {
    let mut file = File::create(output_file)?;
    let mut writer = LzmaWriter::new_compressor(file, 6)?;
    let file_content = get_file_content(input_file)?;
    writer.write(&file_content)?;
    writer.finish()?;

    Ok(())
}
