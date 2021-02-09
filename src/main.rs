mod internals;
use internals::file_explorer_utils::FileManager;
fn main() {
    let mut siv = cursive::default();
    // You can load a theme from a file at runtime for fast development.
    //siv.load_theme_file("assets/style.toml").unwrap();

    let fm = FileManager::new(&mut siv);

    siv.run();
}
