use config::Config;
pub struct FileMangerConfig {
    pub left_panel_initial_path: String,
    pub right_panel_initial_path: String,
}
pub fn read_config() -> FileMangerConfig {
    FileMangerConfig {
        left_panel_initial_path: "/home/artie/Desktop/Left".to_owned(),
        right_panel_initial_path: "/home/artie/Desktop/Right".to_owned(),
    }
}
