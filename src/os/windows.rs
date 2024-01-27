use super::OsBlackhole;
use crate::Blackhole;
use ini::{EscapePolicy, Ini};
use std::{
	ffi::CString,
	os::{raw::c_void, windows::process::CommandExt},
	path::Path,
	process::Command,
};
use windows::{
	core::PCSTR,
	Win32::{
		Storage::FileSystem::{SetFileAttributesA, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM, FILE_FLAGS_AND_ATTRIBUTES},
		System::Threading::CREATE_NO_WINDOW,
		UI::Shell::{SHChangeNotify, SHCNE_UPDATEDIR, SHCNE_UPDATEIMAGE, SHCNF_FLUSH, SHCNF_PATHA},
	},
};

fn set_file_attributes(path: &Path, attrs: FILE_FLAGS_AND_ATTRIBUTES) -> Result<(), std::io::Error> {
	unsafe {
		let path = CString::from_vec_with_nul_unchecked(
			path.as_os_str()
				.as_encoded_bytes()
				.iter()
				.copied()
				.chain(std::iter::once(0))
				.collect::<Vec<_>>(),
		);

		SetFileAttributesA(PCSTR(path.as_ptr() as *const _), attrs)?;
	}

	Ok(())
}

fn notify_dir_changed(path: &Path) {
	unsafe {
		let path = CString::from_vec_with_nul_unchecked(
			path.as_os_str()
				.as_encoded_bytes()
				.iter()
				.copied()
				.chain(std::iter::once(0))
				.collect::<Vec<_>>(),
		);

		SHChangeNotify(SHCNE_UPDATEIMAGE, SHCNF_PATHA | SHCNF_FLUSH, Some(path.as_ptr() as *const c_void), None);
		SHChangeNotify(SHCNE_UPDATEDIR, SHCNF_PATHA | SHCNF_FLUSH, Some(path.as_ptr() as *const c_void), None);
	}
}

fn pin_to_quick_access(path: &Path) -> Result<(), std::io::Error> {
	let script = format!(
		r#"
			$Namespace = 'shell:::{{679f85cb-0220-4080-b29b-5540cc05aab6}}'
			$Blackhole = '{}'

			$QuickAccess = new-object -com shell.application
			$QuickAccessItems = $QuickAccess.Namespace($Namespace).Items()

			$QuickAccessBlackhole = $QuickAccessItems | Where-Object {{$_.Path -EQ $Blackhole}}

			if (-Not $QuickAccessBlackhole) {{
				$QuickAccess.Namespace($Blackhole).Self.InvokeVerb("pintohome")
			}}
		"#,
		path.display().to_string().replace('\'', "''")
	);

	Command::new("powershell")
		.args(["-Command", &script])
		.creation_flags(CREATE_NO_WINDOW.0)
		.stderr(std::process::Stdio::piped())
		.stdout(std::process::Stdio::null())
		.stdin(std::process::Stdio::null())
		.output()
		.and_then(|output| {
			if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
				if stderr.contains("called on MessageLoop that's already been Quit!") || stderr.contains("LoadBitmapFromPngResource") {
					// A silly fix for a silly error (the script still worked)
					return Ok(());
				}
			}

			if output.stderr.is_empty() {
				Ok(())
			} else {
				Err(std::io::Error::new(std::io::ErrorKind::Other, String::from_utf8_lossy(&output.stderr)))
			}
		})
}

fn create_desktop_ini(blackhole: &Path) {
	let mut desktop = Ini::new();

	desktop
		.with_section(Some(".ShellClassInfo"))
		.set("ConfirmFileOp", "0")
		.set("InfoTip", "WARNING: All files stored here will be deleted at the next startup");

	desktop
		.with_section(Some("ViewState"))
		.set("Mode", "")
		.set("Vid", "")
		.set("FolderType", "Generic");

	match std::env::current_exe().map(|p| p.display().to_string()) {
		Ok(exe_path) => {
			desktop
				.with_section(Some(".ShellClassInfo"))
				.set("IconResource", format!("{},0", exe_path));
		}

		Err(err) => {
			log::error!("Failed to get current executable path: {err:?}");
		}
	}

	let ini = blackhole.join("desktop.ini");

	std::fs::remove_file(&ini).ok();

	match desktop.write_to_file_policy(&ini, EscapePolicy::Nothing) {
		Ok(_) => {
			if let Err(err) = set_file_attributes(&ini, FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) {
				log::error!("Failed to set file attributes on desktop.ini: {err:?}");
			}
		}

		Err(err) => log::error!("Failed to write desktop.ini: {err:?}"),
	}
}

impl OsBlackhole for Blackhole {
	fn decorate_blackhole_folder(path: &Path) {
		// Set attributes on BLACKHOLE folder
		if let Err(err) = set_file_attributes(path, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY) {
			log::error!("Failed to set file attributes on BLACKHOLE folder: {err:?}");
		}

		// Create desktop.ini
		create_desktop_ini(path);

		// Tell Explorer the BLACKHOLE folder changed
		notify_dir_changed(path);
	}

	fn chart_blackhole_folder(path: &Path) {
		if let Err(err) = pin_to_quick_access(path) {
			log::error!("Failed to pin BLACKHOLE folder to Quick Access: {err:?}");
		}

		// Tell Explorer that Quick Access changed
		notify_dir_changed(Path::new("shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}"));
	}

	fn should_skip_purge_file(path: &Path) -> bool {
		path.file_name() == Some(OsStr::new("desktop.ini"))
	}
}
