pub mod blackhole {
	use crate::Show;

	// For move_items override
	#[cfg(all(target_os="windows", feature="gui"))] use crate::windows::Windows;

	// For move_n_purge function
	#[cfg(target_os="macos")] use crate::macos::MacOS;

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

	#[cfg(any(not(target_os="windows"), not(feature="gui")))]
	static DIR_MOVE_OPTIONS: fs_extra::dir::CopyOptions = fs_extra::dir::CopyOptions {
		overwrite: false,
		skip_exist: true,
		buffer_size: 64000,
		copy_inside: true,
		content_only: true,
		depth: 0
	};

	#[cfg(any(not(target_os="windows"), not(feature="gui")))]
	static FILE_MOVE_OPTIONS: fs_extra::file::CopyOptions = fs_extra::file::CopyOptions {
		overwrite: false,
		skip_exist: true,
		buffer_size: 64000
	};
	
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
				Err(error) => Show::panic(&format!("Failed to CREATE blackhole directory ({:?}) at {:?}", error, self.path)),
				Ok(_) => println!("Created At: {}", self.path.display())
			}
		}

		fn is_empty(&self) -> Result<bool, io::Error> {
			for entry in self.path.read_dir()? {
				if !EMPTY_DIR_FILTER.contains(&entry?.file_name().to_str().unwrap_or_default()) {
					return Ok(false);
				}
			}
			Ok(true)
		}

		pub fn purge(&self) {
			match self.is_empty() {
				Err(error) => Show::panic(&format!("Failed to READ blackhole directory ({:?}) at {:?}", error, self.path)),
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
				Err(error) => { Show::panic(&format!("Failed to PURGE blackhole directory ({:?}) at {:?}", error, self.path)); return },
				Ok(_) => return
			};

			match trash::delete(&self.path) {
				Err(error) => Show::panic(&format!("Failed to PURGE blackhole directory ({:?}) at {:?}", error, self.path)),
				Ok(_) => ()
			}

			println!("Purged Blackhole directory!");
			
			self.create();
		}

		pub fn send(&self, path_str: OsString) {
			let path = Path::new(&path_str);
			if !path.exists() || path.file_name().is_none() {
				Show::panic(&format!("File does not exist! ({:?})", path));
				return;
			}

			if path.starts_with(&self.path) {
				Show::panic(&String::from("This file or folder is already in your Blackhole."));
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
					Show::panic(&format!("File already exists: {:?}", blackhole_send_path));
					return;
				}

				// If the file does exist, we need to pop it so we can push "$file_name ($i++)" into it in the next loop iteration
				blackhole_send_path.pop();
			}

			if !path.is_file() && !path.is_dir() {
				Show::panic(&format!("Failed to move file/folder to Blackhole (\"Path provided was not a file or directory\")\n{:?} -> {:?}", path, &blackhole_send_path));
				return;
			}

			match self.move_items(path, &blackhole_send_path) {
				Ok(_) => {},
				Err(error) => Show::panic(&format!("Failed to move file/folder to Blackhole ({:?})\n{:?} -> {:?}", error, path, &blackhole_send_path)),
			}
		}
		
		// If we're not using Windows (the Windows trait will override this to use Windows Explorer's move files command)
		// or gui feature is not enabled, we'll use fs_extra::move_items to move the items. Not ideal as it doesn't show
		// progress, but it's cross-platform.
		// TODO: Native MacOS move_items function	
		// FIXME: https://github.com/webdesus/fs_extra/issues/39	
		#[cfg(any(not(target_os="windows"), not(feature="gui")))]
		fn move_items(&self, from: &Path, to: &Path) -> Result<(), String> {
			/*let mut renamed_from = from.to_owned();
			if from.file_name().unwrap() != to.file_name().unwrap() {
				// move_dir doesn't rename directories when moving, so we'll have to do it ourselves
				renamed_from.pop();
				renamed_from.push(&to.file_name().unwrap());
				match fs::rename(from, &renamed_from) {
					Err(error) => { return Err(String::from(format!("{:?}", &error))) },
					Ok(_) => {}
				}
			}*/

			match match from.is_dir() {
				true => fs_extra::dir::move_dir(from, to, &DIR_MOVE_OPTIONS),
				false => fs_extra::file::move_file(from, to, &FILE_MOVE_OPTIONS)
			} {
				Err(error) => Err(String::from(format!("{:?}", &error))),
				Ok(_) => Ok(())
			}
		}
	}
}