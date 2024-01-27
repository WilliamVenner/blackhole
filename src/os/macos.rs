embed_plist::embed_launchd_plist!("../../assets/launchd.plist");

use super::OsBlackhole;
use crate::Blackhole;
use std::{
	ffi::{CStr, CString, OsStr},
	os::{
		raw::{c_char, c_void},
		unix::ffi::{OsStrExt, OsStringExt},
	},
	path::Path,
};

#[link(name = "blackhole_macos_objc", kind = "static")]
extern "C" {
	fn objc_set_blackhole_icon(blackhole_path: *const c_char, icon_path: *const c_char) -> *const c_char;
	fn objc_add_blackhole_to_favorites(blackhole_path: *const c_char) -> *const c_char;
}

fn set_blackhole_icon(path: &Path) -> Result<(), std::io::Error> {
	let icon_path = CString::new({
		let mut icon_path = std::env::current_exe()?;
		icon_path.pop();
		icon_path.pop();
		icon_path.push("Resources/blackhole.icns");

		if !icon_path.is_file() {
			return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Icon file not found"));
		}

		icon_path.into_os_string().into_vec()
	})
	.map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

	let blackhole_path = CString::new(path.as_os_str().as_bytes()).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

	let err = unsafe { objc_set_blackhole_icon(blackhole_path.as_ptr(), icon_path.as_ptr()) };
	if err.is_null() {
		Ok(())
	} else {
		let err_str = unsafe { CStr::from_ptr(err) }.to_string_lossy();
		unsafe { libc::free(err as *mut c_void) };
		Err(std::io::Error::new(std::io::ErrorKind::Other, err_str))
	}
}

fn add_blackhole_to_favorites(path: &Path) -> Result<(), std::io::Error> {
	let blackhole_path = CString::new(path.as_os_str().as_bytes()).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

	let err = unsafe { objc_add_blackhole_to_favorites(blackhole_path.as_ptr()) };
	if err.is_null() {
		Ok(())
	} else {
		let err_str = unsafe { CStr::from_ptr(err) }.to_string_lossy();
		unsafe { libc::free(err as *mut c_void) };
		Err(std::io::Error::new(std::io::ErrorKind::Other, err_str))
	}
}

// Register Blackhole to start at login
fn launchd() -> Result<(), std::io::Error> {
	let mut plist_path = dirs::home_dir().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "No home directory"))?;

	plist_path.push("Library/LaunchAgents");

	std::fs::create_dir_all(&plist_path)?;

	plist_path.push("com.venner.blackhole.plist");

	let exe_path = std::env::current_exe()?
		.into_os_string()
		.into_string()
		.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Executable path is not UTF-8"))?;

	let launchd_plist = embed_plist::get_launchd_plist();
	let launchd_plist = std::str::from_utf8(launchd_plist).expect("Expected launchd.plist to be UTF-8 encoded");
	let launchd_plist = launchd_plist.replacen("$BLACKHOLE_EXE_PATH", quick_xml::escape::escape(&exe_path).as_ref(), 1);

	if std::fs::read(&plist_path).ok().as_deref() == Some(launchd_plist.as_bytes()) {
		// Already installed
		Ok(())
	} else {
		std::fs::write(plist_path, launchd_plist.into_bytes())
	}
}

impl OsBlackhole for Blackhole {
	fn decorate_blackhole_folder(path: &Path) {
		if let Err(err) = set_blackhole_icon(path) {
			log::error!("Failed to set icon on BLACKHOLE folder: {err:?}");
		}
	}

	fn chart_blackhole_folder(path: &Path) {
		if let Err(err) = add_blackhole_to_favorites(path) {
			log::error!("Failed to add BLACKHOLE folder to favorites: {err:?}");
		}

		if let Err(err) = launchd() {
			log::error!("Failed to register Blackhole to start at login: {err:?}");
		}
	}

	fn should_skip_purge_file(path: &Path) -> bool {
		path.file_name()
			.is_some_and(|file_name| [OsStr::new(".DS_Store"), OsStr::new("Icon\r")].contains(&file_name))
	}
}
