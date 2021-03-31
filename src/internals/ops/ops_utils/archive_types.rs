use phf::phf_set;
pub static ARCHIVES_TYPES: phf::Set<&'static str> = phf_set! {
    "application/x-archive",
"application/x-cpio",
"application/x-shar",
"application/x-iso9660-image",
"application/x-sbx",
"application/x-tar",
"application/x-bzip2",
"application/gzip",
"application/x-lzip",
"application/x-lzma",
"application/x-lzop",
"application/x-snappy-framed",
"application/x-xz",
"application/x-compress",
"application/zstd",
"application/x-7z-compressed",
"application/x-ace-compressed",
"application/x-astrotite-afa",
"application/x-alz-compressed",
"application/vnd.android.package-archive",
"application/octet-stream",
"application/x-freearc",
"application/x-arj",
"application/x-b1",
"application/vnd.ms-cab-compressed",
"application/x-cfs-compressed",
"application/x-dar",
"application/x-dgc-compressed",
"application/x-apple-diskimage",
"application/x-gca-compressed",
"application/java-archive",
"application/x-lzh",
"application/x-lzx",
"application/x-rar-compressed",
"application/x-stuffit",
"application/x-stuffitx",
"application/x-gtar",
"application/x-ms-wim",
"application/x-xar",
"application/zip",
"application/x-zoo",
};