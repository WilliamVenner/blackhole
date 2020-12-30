#![cfg_attr(feature="gui", windows_subsystem = "windows")]

use std::env;

mod show;
use show::show::Show;

mod blackhole;
use blackhole::blackhole::Blackhole;

#[cfg(target_os="windows")] mod windows;
#[cfg(target_os="linux")] mod linux;
#[cfg(target_os="macos")] mod macos;

#[cfg(feature="gui")]
use opener::open;

fn main() {
	let should_purge: bool = env::args_os().any(|arg| arg == "--purge");
	match Blackhole::new(should_purge) {
		Ok(blackhole) => {
			println!("Location: {}", blackhole.path.display());

			if !should_purge {
				#[cfg(feature="gui")]
				open(&blackhole.path).ok();

				Show::msg(&String::from("Blackhole directory initialized!"));
			}
		},
		Err(error) => { Show::panic(&String::from(error)); }
	}
}
