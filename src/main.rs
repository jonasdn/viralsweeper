extern crate gio;
extern crate gtk;

use gio::prelude::*;

use std::env;

fn main() {
    let app = gtk::Application::new(Some("org.miner"), gio::ApplicationFlags::FLAGS_NONE)
        .expect("Application::new failed");
    app.connect_activate(|a| {
        viralsweeper::run(a);
    });
    app.run(&env::args().collect::<Vec<_>>());
}
