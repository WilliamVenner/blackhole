#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logs;

use os::OsBlackhole;
use std::{ffi::OsStr, path::PathBuf};
use uuid::Uuid;

mod os {
	use std::path::Path;

	#[cfg(windows)]
	pub mod windows;
	#[cfg(windows)]
	pub use windows::*;

	#[cfg(target_os = "macos")]
	pub mod macos;
	#[cfg(target_os = "macos")]
	pub use macos::*;

	pub trait OsBlackhole {
		/// Decorate the Blackhole folder: give it an icon, and set any special attributes.
		fn decorate_blackhole_folder(path: &Path);

		/// "Chart" the Blackhole folder: add it to "easy access" locations.
		fn chart_blackhole_folder(path: &Path);

		/// Should this file be skipped when purging the Blackhole folder?
		fn should_skip_purge_file(path: &Path) -> bool;

		/// Sends a file or directory to the Blackhole folder.
		fn send(&self, path: &Path) -> Result<(), std::io::Error>;
	}
}

#[derive(Debug, Clone, Copy)]
enum Operation {
	Initialize,
	Purge,
	SendTo,
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

			if path == purge_dir || Self::should_skip_purge_file(&path) {
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

fn run() -> Result<i32, std::io::Error> {
	let mut args = std::env::args_os().skip(1);
	let args_len = args.len();
	let operation = args.next();
	let operation = match (operation.as_deref(), args_len) {
		(None, 0) => Operation::Initialize,

		(arg, 1) if arg == Some(OsStr::new("--purge")) => Operation::Purge,

		(arg, 2) if arg == Some(OsStr::new("--send")) => Operation::SendTo,

		_ => {
			eprintln!("Usage: blackhole [--purge | --send path]");
			return Ok(1);
		}
	};

	logs::init(operation);
	log::info!(concat!("Blackhole v", env!("CARGO_PKG_VERSION")));

	let mut blackhole = Blackhole::new()?;

	match operation {
		Operation::Initialize => {
			log::info!("Opening Blackhole");
			blackhole.init_and_open()?;
		}

		Operation::Purge => {
			log::info!("Purging Blackhole");
			blackhole.purge()?;
		}

		Operation::SendTo => {
			let path = PathBuf::from(args.next().unwrap());

			log::info!("Sending {} to the Blackhole", path.display());

			blackhole.send(&path)?;
		}
	}

	Ok(0)
}

fn main() -> Result<(), std::io::Error> {
	std::process::exit(run()?)
}
