use std::process::Command;

use cursive::event::*;
use cursive::CursiveExt;
pub fn open(siv: &mut cursive::Cursive) {
    let output = Command::new("nano")
        //.stdout(Stdio::null())
        //.arg("file.txt")
        //.spawn()
        //.output()
        .status()
        .expect("failed to execute process");
    siv.add_global_callback(Key::F3, open);
    siv.run();
}
