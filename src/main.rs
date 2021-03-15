mod internals;
use internals::file_manager::FileManager;

fn main() {
    let mut siv = cursive::default();
    // You can load a theme from a file at runtime for fast development.
    //siv.load_theme_file("assets/style.toml").unwrap();

    FileManager::new(&mut siv);

    siv.run();
}
