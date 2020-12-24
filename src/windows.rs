use crate::Blackhole;
use crate::Show;

use std::{iter, fs, os::windows::ffi::OsStrExt, process::Command};
use ini::{Ini, EscapePolicy};
use winreg::{enums::*, RegKey};
use winapi;

pub trait Windows {
	fn powershell(script: String);
	fn quick_access(path: &std::path::PathBuf);
	fn set_file_attributes(path: &std::path::PathBuf, attr: winapi::shared::minwindef::DWORD);
	fn set_blackhole_attributes(path: &std::path::PathBuf);
	fn edit_startup_registry();
	fn chores(&self);
}

impl Windows for Blackhole {
	// Run a PowerShell script
	// TODO: Replace these with winapi calls when the relevant functions are binded
	fn powershell(script: String) {
		Command::new("powershell").args(&["-Command", &script]).status().ok();
	}

	// Add $BLACKHOLE to the Quick Access tab
	fn quick_access(path: &std::path::PathBuf) {
		Blackhole::powershell(format!("$o = new-object -com shell.application\n$o.Namespace('{}').Self.InvokeVerb('pintohome')", path.display()));
	}

	// Sets Windows file attributes
	fn set_file_attributes(path: &std::path::PathBuf, attr: winapi::shared::minwindef::DWORD) {
		let path_utf16: Vec<u16> = path.as_os_str().encode_wide().chain(iter::once(0)).collect();
		unsafe {
			winapi::um::fileapi::SetFileAttributesW(path_utf16.as_ptr(), attr);
		}
	}
	
	// Configures the $BLACKHOLE folder
	fn set_blackhole_attributes(path: &std::path::PathBuf) {
		// Set the file attributes of the blackhole directory itself
		Blackhole::set_file_attributes(path, winapi::um::winnt::FILE_ATTRIBUTE_SYSTEM | winapi::um::winnt::FILE_ATTRIBUTE_READONLY);

		// Create desktop.ini
		let mut ini = path.to_owned();
		ini.push("desktop.ini");
		
		if ini.exists() {
			match fs::remove_file(&ini) {
				Ok(_) => (),
				Err(error) => {
					println!("Error deleting desktop.ini: {}", error);
				}
			};
		}

		let mut desktop = Ini::new();
		desktop.with_section(Some(".ShellClassInfo"))
			.set("ConfirmFileOp", "0")
			.set("InfoTip", "WARNING: All files stored here will be deleted at the next startup");

		desktop.with_section(Some("ViewState"))
			.set("Mode", "")
			.set("Vid", "")
			.set("FolderType", "Generic");
		
		// Copy the icon from the exe over to the blackhole directory
		match std::env::current_exe() {
			Ok(exe_path) => {
				match exe_path.into_os_string().into_string() {
					Ok(exe_path) => { desktop.with_section(Some(".ShellClassInfo")).set("IconResource", format!("{},0", exe_path)); }
					Err(_) => {
						println!("Error converting executable path string");
						return;
					}
				}
			},
			Err(error) => {
				println!("Error getting executable path: {}", error);
				return;
			}
		}
		
		// Write desktop.ini and set its file attributes
		let escape_policy : ini::EscapePolicy = EscapePolicy::Nothing;
		match desktop.write_to_file_policy(&ini, escape_policy) {
			Ok(_) => { Blackhole::set_file_attributes(&ini, winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN | winapi::um::winnt::FILE_ATTRIBUTE_SYSTEM); }
			Err(error) => {
				println!("Error setting file attributes on desktop.ini: {}", error);
				return;
			}
		}
	}

	fn edit_startup_registry() {
		// Add blackhole.exe --purge to the startup registry
		let exe_path = match std::env::current_exe() {
			Ok(exe_path) => exe_path,
			Err(error) => {
				Show::error(&format!("Error getting executable path: {}", error));
				return;
			}
		};

		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let startup = match hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE) {
			Ok(startup) => startup,
			Err(error) => {
				Show::error(&format!("Error opening registry subkey: {}", error));
				return;
			}
		};

		match startup.set_value("Blackhole", &format!("{} --purge", exe_path.display())) {
			Ok(_) => return,
			Err(error) => {
				Show::error(&format!("Error setting registry key: {}", error));
				return;
			}
		};
	}

	fn chores(&self) {
		// If we're running Windows, add blackhole.exe --purge to the startup registry
		Blackhole::edit_startup_registry();
		
		// Set file/folder attributes
		Blackhole::set_blackhole_attributes(&self.path);

		// Add $BLACKHOLE to Quick Access links
		Blackhole::quick_access(&self.path);
	}
}