use os::OsBlackhole;
use std::{ffi::OsStr, path::PathBuf};
use uuid::Uuid;

mod os {
	#[cfg(windows)]
	pub mod windows;
	use std::path::Path;

	#[cfg(windows)]
	pub use windows::*;

	pub trait OsBlackhole {
		/// Decorate the Blackhole folder: give it an icon, and set any special attributes.
		fn decorate_blackhole_folder(path: &Path);

		/// "Chart" the Blackhole folder: add it to "easy access" locations.
		fn chart_blackhole_folder(path: &Path);
	}
}

pub struct Blackhole {
	pub path: PathBuf,
}
impl Blackhole {
	pub fn new() -> Result<Self, std::io::Error> {
		Ok(Self {
			path: std::env::var_os("BLACKHOLE_DIR")
				.map(PathBuf::from)
				.or_else(|| dirs::home_dir().map(|dir| dir.join("BLACKHOLE")))
				.ok_or_else(|| {
					std::io::Error::new(
						std::io::ErrorKind::Unsupported,
						"No home directory or BLACKHOLE_DIR environment variable set",
					)
				})?,
		})
	}

	pub fn init_and_open(&mut self) -> Result<(), std::io::Error> {
		match std::fs::create_dir(&self.path) {
			Ok(_) => {}
			Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
			Err(err) => return Err(err),
		}

		Self::decorate_blackhole_folder(&self.path);

		Self::chart_blackhole_folder(&self.path);

		opener::open(&self.path).map_err(|err| match err {
			opener::OpenError::Io(err) => err,
			err => std::io::Error::new(std::io::ErrorKind::Other, err),
		})?;

		Ok(())
	}

	pub fn purge(&mut self) -> Result<(), std::io::Error> {
		if !self.path.exists() {
			// Don't bother if we don't have a Blackhole to purge.
			log::info!("No Blackhole to purge");
			return Ok(());
		}

		// Create a temporary "purge" Blackhole inside the actual Blackhole.
		let purge_dir = self.path.join(format!("BLACKHOLE_PURGE_{}", Uuid::new_v4()));

		log::info!("Creating purge directory at {}", purge_dir.display());

		std::fs::create_dir_all(&purge_dir)?;

		log::info!("Moving contents of Blackhole to purge directory");

		// Move the contents of the actual Blackhole into the purge Blackhole.
		let mut empty = true;
		for entry in std::fs::read_dir(&self.path)? {
			let entry = entry?;
			let path = entry.path();

			if path == purge_dir || path.file_name() == Some(OsStr::new("desktop.ini")) {
				continue;
			}

			let purge_path = purge_dir.join(
				path.strip_prefix(&self.path)
					.expect("Expected path to be a child of the Blackhole directory"),
			);

			if let Err(err) = std::fs::rename(&path, &purge_path) {
				log::error!(
					"Failed to move {} to the purge directory at {} ({})",
					path.display(),
					purge_path.display(),
					err
				);
			} else {
				empty = false;
			}
		}

		if !empty {
			log::info!("Moving to trash");

			// Rename the purge Blackhole.
			let purge_dir = {
				let new_purge_dir = purge_dir.with_file_name("BLACKHOLE");
				std::fs::rename(&purge_dir, &new_purge_dir)?;
				new_purge_dir
			};

			// Decorate it so that it looks like a Blackhole folder.
			Self::decorate_blackhole_folder(&purge_dir);

			// Move the purge Blackhole to the trash.
			trash::delete(purge_dir).map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
		} else {
			log::info!("Blackhole was empty, aborting");

			// Remove the empty purge Blackhole.
			if let Err(err) = std::fs::remove_dir(&purge_dir) {
				log::error!("Failed to remove empty purge directory: {err:?}");
			}

			Ok(())
		}
	}
}

fn main() -> Result<(), std::io::Error> {
	std::env::set_var("RUST_LOG", "blackhole=info");
	env_logger::init();
	log::set_max_level(log::LevelFilter::Info);

	log::info!(concat!("Blackhole v", env!("CARGO_PKG_VERSION")));

	let mut blackhole = Blackhole::new()?;
	let purge = std::env::args_os().nth(1).as_deref() == Some(OsStr::new("--purge"));
	if purge {
		log::info!("Purging Blackhole");
		blackhole.purge()
	} else {
		log::info!("Opening Blackhole");
		blackhole.init_and_open()
	}
}
