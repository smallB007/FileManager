mod internals;
use internals::file_explorer_utils::FileManager;
fn main() {
    let mut siv = cursive::default();
    let fm = FileManager::new(&mut siv);
    siv.run();
}
