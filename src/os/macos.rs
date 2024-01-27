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

impl OsBlackhole for Blackhole {
	fn decorate_blackhole_folder(path: &Path) {
		if let Err(err) = set_blackhole_icon(path) {
			log::error!("Failed to set icon on BLACKHOLE folder: {err:?}");
		}
	}

	fn chart_blackhole_folder(path: &Path) {}

	fn should_skip_purge_file(path: &Path) -> bool {
		path.file_name()
			.is_some_and(|file_name| [OsStr::new(".DS_Store"), OsStr::new("Icon\r")].contains(&file_name))
	}
}
