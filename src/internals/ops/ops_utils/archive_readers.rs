use phf::phf_map;
use std::{
    fs::{self},
    io,
};
pub enum EarchivesReaders {
    EZipReader(&'static str),
}
pub static ARCHIVES_READERS_TYPES: phf::Map<&'static str, EarchivesReaders> = phf_map! {
    "application/zip"=>EarchivesReaders::EZipReader("application/zip"/*todo possibly not needed*/),
};
pub trait ArchiveReader {
    fn read(&self, archive: &std::path::Path) -> Vec<String>;
}
pub struct ArchiveReaderFactory {}

impl ArchiveReaderFactory {
    pub fn create_reader(reader_type: &EarchivesReaders) -> Box<dyn ArchiveReader> {
        match reader_type {
            EarchivesReaders::EZipReader(some_string) => Box::new(ZipArchiveReader {}),
        }
    }
}

pub struct ZipArchiveReader {}

impl ArchiveReader for ZipArchiveReader {
    fn read(&self, archive: &std::path::Path) -> Vec<String> {
        let mut res = Vec::new();
        let fname = archive; //std::path::Path::new(archive);

        let file = fs::File::open(&fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            match file.enclosed_name() {
                Some(path) => {
                    //println!("{:?}", path);
                    match path.to_str() {
                        Some(path_str) => {
                            res.push(path_str.to_owned());
                        }
                        None => {}
                    }
                }
                None => {
                    println!("None enclosed");
                    continue;
                }
            }
        }
        res
    }
}

fn real_main(archive: &std::path::Path) -> i32 {
    /*let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        return 1;
    }*/
    let fname = archive; //std::path::Path::new(archive);

    let file = fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => {
                println!("{:?}", path);
                path.to_owned()
            }
            None => {
                println!("None enclosed");
                continue;
            }
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        /* // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }*/
    }
    return 0;
}
