use std::io::prelude::*;
use std::{fs::File, path::PathBuf};
use tar::Builder;

use crate::internals::ops::ops_utils::lzma::create_xz_archive;

pub fn create_tar_archive(input: &Vec<String>) {
    let output = "/home/artie/Desktop/Left/archive.tar";
    let file = File::create(output).unwrap();
    let mut arch = Builder::new(file);
    for path in input {
        let path_b = PathBuf::from(path);
        let path_name = path_b.file_name().unwrap().to_str().unwrap();

        arch.append_file(path_name, &mut File::open(&path_b).unwrap()).unwrap();
    }
    create_xz_archive(output, "/home/artie/Desktop/Left/output_file.tar.xz");
    /*arch.append_path("file1.txt").unwrap();
    arch.append_file("file2.txt", &mut File::open("file3.txt").unwrap())
        .unwrap();*/
}
