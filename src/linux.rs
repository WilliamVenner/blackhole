use crate::Blackhole;
use crate::Show;

use std::{env, fs};

pub trait Linux {
	fn copy_executable();
	fn chores(&self);
}

impl Linux for Blackhole {
	fn copy_executable() {
		// Locate the executable folder and symlink us there
		let mut new_exe_path = match dirs::executable_dir() {
			Some(new_exe_path) => new_exe_path,
			None => return
		};

		new_exe_path.push("blackhole");

		let exe_path = match env::current_exe() {
			Ok(exe_path) => exe_path,
			Err(error) => {
				Show::panic(&format!("Error getting executable path: {}", error));
				return;
			}
		};

		if exe_path == new_exe_path { return }

		match fs::rename(&exe_path, &new_exe_path) {
			Ok(_) => (),
			Err(error) => {
				Show::panic(&format!("Error moving executable to executables path: {}", error));
				return;
			}
		}

		Show::msg(&format!("Blackhole executable has been moved to {}", new_exe_path.display()));
	}

	fn chores(&self) {
		// If we're running Linux, copy the executable to the executables folder
		Blackhole::copy_executable();
	}
}