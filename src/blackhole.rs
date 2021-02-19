pub mod blackhole {
	use crate::Show;

	use std::{ffi::OsString, fs, io};
	use std::path::{Path, PathBuf};
	use dirs;
	use trash;

	#[cfg(target_os="windows")]
	static EMPTY_DIR_FILTER: [&str; 1] = ["desktop.ini"];
	#[cfg(target_os="macos")]
	static EMPTY_DIR_FILTER: [&str; 2] = [".DS_Store", "Icon\r"];
	#[cfg(not(any(target_os="windows", target_os="macos")))]
	static EMPTY_DIR_FILTER: [&str; 0] = [];

	#[cfg(all(target_os="windows", feature="gui"))] use crate::windows::Windows;
	
	pub struct Blackhole {
		pub path: PathBuf
	}

	impl Blackhole {
		pub fn new() -> Result<Blackhole, &'static str> {
			let mut home_dir = match dirs::home_dir() {
				Some(home_dir) => home_dir,
				None => { return Err("Could not find a home directory!") },
			};

			home_dir.push("$BLACKHOLE");
			
			let new = Blackhole { path: home_dir };

			// If the blackhole directory doesn't exist, create it
			new.create();
			println!("Blackhole initialized!");

			Ok(new)
		}
		
		fn create(&self) {
			if self.path.is_dir() { return };

			match fs::create_dir(&self.path) {
				Err(error) => Show::panic(&format!("Failed to CREATE blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => println!("Created At: {}", self.path.display())
			}
		}

		fn empty(&self) -> Result<bool, io::Error> {
			for entry in self.path.read_dir()? {
				if !EMPTY_DIR_FILTER.contains(&entry?.file_name().to_str().unwrap_or_default()) {
					return Ok(false);
				}
			}
			Ok(true)
		}

		#[cfg(target_os="macos")]
		fn move_n_purge(&self) -> Result<bool, io::Error> {
			let mut temp_blackhole = self.path.to_owned();
			temp_blackhole.push("$BLACKHOLE");

			if temp_blackhole.is_file() {
				match trash::delete(&temp_blackhole) {
					Err(error) => Show::panic(&format!("Failed to delete $BLACKHOLE/$BLACKHOLE (please don't create a $BLACKHOLE file inside your blackhole? You are messing with space-time) (\"{:?}\") at {:?}", error, self.path)),
					Ok(_) => ()
				}
			}

			// If the temporary blackhole already exists, move it to the trash first
			if temp_blackhole.is_dir() {
				match trash::delete(&temp_blackhole) {
					Err(error) => Show::panic(&format!("Failed to delete restored(?) $BLACKHOLE/$BLACKHOLE directory (\"{:?}\") at {:?}", error, self.path)),
					Ok(_) => ()
				}
			}

			match fs::create_dir(&temp_blackhole) {
				Err(error) => Show::panic(&format!("Failed to create temporary $BLACKHOLE/$BLACKHOLE directory during purge (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => ()
			}

			println!("Moving files...");

			// Move the files into the temporary blackhole
			let temp_blackhole_name: OsString = OsString::from("$BLACKHOLE");
			for entry in self.path.read_dir()? {
				let file_path = entry?.path();
				match file_path.file_name() {
					None => continue,
					Some(file_name) => {
						if file_path.is_dir() && file_name == temp_blackhole_name { continue; }
						temp_blackhole.push(&file_name);
						fs::rename(&file_path, &temp_blackhole).ok();
						temp_blackhole.pop();
					}
				}
			}

			// Give it an icon
			Blackhole::set_blackhole_icon(&temp_blackhole);
			
			// Finally, let's delete it
			match trash::delete(&temp_blackhole) {
				Err(error) => Show::panic(&format!("Failed to delete temporary $BLACKHOLE/$BLACKHOLE directory during purge (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => ()
			}

			println!("Purged Blackhole directory!");

			Ok(true)
		}

		pub fn purge(&self) {
			match self.empty() {
				Err(error) => Show::panic(&format!("Failed to READ blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(empty) => {
					if empty {
						println!("Blackhole directory empty.");
						return
					}
				}
			}

			// On MacOS, it is not possible to add folders to the "Favourites" sidebar in Finder because Apple deprecated the API and provided no alternative.
			// So that the user can still pin the Blackhole to their Favourites, we move all the contents into a new $BLACKHOLE directory which is then moved to the trash instead.
			#[cfg(target_os="macos")]
			match self.move_n_purge() {
				Err(error) => { Show::panic(&format!("Failed to PURGE blackhole directory (\"{:?}\") at {:?}", error, self.path)); return },
				Ok(_) => return
			};

			match trash::delete(&self.path) {
				Err(error) => Show::panic(&format!("Failed to PURGE blackhole directory (\"{:?}\") at {:?}", error, self.path)),
				Ok(_) => ()
			}

			println!("Purged Blackhole directory!");
			
			self.create();
		}

		pub fn send(&self, path_str: OsString) {
			let path = Path::new(&path_str);
			if !path.exists() || path.file_name().is_none() {
				Show::panic(&format!("File does not exist! (\"{:?}\")", path));
				return;
			}

			let file_name = path.file_name().unwrap();

			let mut blackhole_send_path = self.path.to_owned();
			let mut i: u32 = 0;
			loop {
				if i == 0 {
					blackhole_send_path.push(file_name);
				} else {
					let mut file_name_dupe = file_name.to_os_string();
					file_name_dupe.push(&format!(" ({})", i));
					blackhole_send_path.push(&file_name_dupe);
				}

				if !blackhole_send_path.exists() {
					break;
				}

				i += 1;
				if i >= u32::MAX {
					Show::panic(&format!("File already exists: \"{:?}\"", blackhole_send_path));
					return;
				}

				blackhole_send_path.pop();
			}

			match Blackhole::move_items(path, &blackhole_send_path) {
				Ok(_) => {},
				Err(error) => Show::panic(&format!("Failed to move file/folder to Blackhole (\"{:?}\")\n{:?} -> {:?}", error, path, &blackhole_send_path)),
			}
		}
		
		#[cfg(any(not(target_os="windows"), not(feature="gui")))]
		fn move_items(from: &Path, to: &Path) -> Result<(), String> {
			match fs_extra::move_items(&vec![from], to, &fs_extra::dir::CopyOptions::new()) {
				Err(error) => Err(String::from(format!("{:?}", &error))),
				Ok(_) => Ok(())
			}
		}
	}
}