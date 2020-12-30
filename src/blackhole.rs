pub mod blackhole {
	use crate::Show;

	use std::{collections::HashSet, fs, io};
	use dirs;
	use trash;
	use lazy_static::lazy_static;

	#[cfg(target_os = "windows")]
	lazy_static! { static ref EMPTY_DIR_FILTER: HashSet<&'static str> = HashSet::from(["desktop.ini"].iter().cloned().collect()); }
	#[cfg(target_os = "macos")]
	lazy_static! { static ref EMPTY_DIR_FILTER: HashSet<&'static str> = [".DS_Store", "Icon\r"].iter().cloned().collect(); }
	#[cfg(not(any(target_os = "windows", target_os = "macos")))]
	lazy_static! { static ref EMPTY_DIR_FILTER: HashSet<&'static str> = return HashSet::new(); }
	
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
			#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
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
			let found_file = self.path.read_dir()?.into_iter().filter(
				|entry| !entry.is_err() &&
				!EMPTY_DIR_FILTER.contains(
					entry.as_ref().unwrap().file_name().to_str().unwrap_or_default()
				)
			).next();
			
			if found_file.is_none() {
				Ok(true)
			} else {
				let found_file = found_file.unwrap();
				if found_file.is_ok() {
					println!("Found file: {}", found_file.unwrap().path().to_str().unwrap_or_default());
				}
				Ok(false)
			}
		}

		#[cfg(target_os = "macos")]
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

			// On MacOS, it is not possible to add folders to the "Favourites" sidebar in Finder because Apple deprecated the API and provided no alternative.
			// So that the user can still pin the Blackhole to their Favourites, we move all the contents into a new $BLACKHOLE directory which is then moved to the trash instead.
			#[cfg(target_os = "macos")]
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
	}

	#[cfg(target_os = "windows")] use crate::windows::Windows;
	#[cfg(target_os = "linux")]   use crate::linux::Linux;
	#[cfg(target_os = "macos")]   use crate::macos::MacOS;
}