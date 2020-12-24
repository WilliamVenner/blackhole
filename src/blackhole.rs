pub mod blackhole {
	use crate::Show;

	use std::{ffi::OsString, fs, io};
	use dirs;
	use trash;

	pub struct Blackhole {
		pub path: std::path::PathBuf
	}

	impl Blackhole {
		pub fn new(should_purge: bool) -> Result<Blackhole, &'static str> {
			let mut home_dir = match dirs::home_dir() {
				Some(home_dir) => home_dir,
				None => { return Err("Could not find a home directory!") },
			};

			home_dir.push("$BLACKHOLE");
			
			let new = Blackhole { path: home_dir };

			// If the blackhole directory doesn't exist, create it
			new.create();
			println!("Blackhole initialized!");

			// If it does exist & we've started up in purge mode, delete it
			if should_purge { new.purge() }
			
			// Run any chores
			#[cfg(any(windows, linux))]
			new.chores();

			Ok(new)
		}
		
		fn create(&self) {
			if self.path.is_dir() { return };

			match fs::create_dir(&self.path) {
				Err(error) => Show::panic(&format!("Failed to CREATE blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => return
			}
		}

		fn empty(&self) -> Result<bool, io::Error> {
			if !cfg!(windows) { return Ok(self.path.read_dir()?.next().is_none()) }

			let desktop_ini: OsString = OsString::from("desktop.ini");
			for entry in fs::read_dir(&self.path)? {
				match entry?.path().file_name() {
					None => continue,
					Some(file_name) => {
						if file_name != desktop_ini {
							return Ok(false);
						}
					}
				}
			}
			
			Ok(true)
		}

		fn purge(&self) {
			match self.empty() {
				Err(error) => Show::panic(&format!("Failed to READ blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(empty) => {
					if empty {
						println!("Blackhole directory empty.");
						return
					}
				}
			}

			match trash::delete(&self.path) {
				Err(error) => Show::panic(&format!("Failed to PURGE blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => ()
			}

			println!("Purged Blackhole directory!");
			
			self.create();
		}
	}

	#[cfg(windows)] use crate::windows::Windows;
	#[cfg(linux)] use crate::linux::Linux;
}