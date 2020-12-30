#![cfg_attr(feature="gui", windows_subsystem = "windows")]

use std::env;

mod show;
use show::show::Show;

mod blackhole;
use blackhole::blackhole::Blackhole;

#[cfg(target_os = "windows")] mod windows;
#[cfg(target_os = "linux")] mod linux;
#[cfg(target_os = "macos")] mod macos;

// TODO open blackhole after initialization

fn main() {
    let should_purge: bool = env::args_os().any(|arg| arg == "--purge");
    match Blackhole::new(should_purge) {
        Ok(blackhole) => {
            if !should_purge { Show::msg(&String::from("Blackhole directory initialized!")); }
            println!("Location: {}", blackhole.path.display())
        },
        Err(error) => { Show::panic(&String::from(error)); }
    }
}
