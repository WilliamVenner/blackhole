use crate::Blackhole;
use crate::Show;

use std::{ffi::{CString, OsString}, fs, iter, os::windows::ffi::OsStrExt, os::windows::process::CommandExt, path::Path, process::Command};
use ini::{Ini, EscapePolicy};
use winreg::{enums::*, RegKey};
use winapi::{
	self,
	um::winbase::CREATE_NO_WINDOW,
	shared::minwindef::UINT,
	shared::windef::HWND,
	shared::winerror::S_OK,
	um::shellapi::{SHFileOperationW, FOF_ALLOWUNDO, FO_MOVE, SHFILEOPSTRUCTW},
	um::winnt::PCZZWSTR,
};

// Bind SHChangeNotify
extern "system" {
	pub fn SHChangeNotify(
		wEventId: winapi::um::winnt::LONG,
		uFlags: winapi::shared::minwindef::UINT,
		dwItem1: winapi::shared::minwindef::LPCVOID,
		dwItem2: winapi::shared::minwindef::LPCVOID
	);
}
const SHCNE_UPDATEDIR: winapi::um::winnt::LONG = 0x00001000;
const SHCNF_PATHW: winapi::shared::minwindef::UINT = 0x0005;

pub trait Windows {
	fn powershell(script: String);
	fn change_notify(ptr: *const winapi::ctypes::c_void);
	fn set_file_attributes(path: &std::path::PathBuf, attr: winapi::shared::minwindef::DWORD) -> *const u16;
	fn quick_access(path: &std::path::PathBuf);
	fn set_blackhole_attributes(path: &std::path::PathBuf);
	fn edit_context_menu_registry();
	fn edit_startup_registry();
	fn move_items(from: &Path, to: &Path) -> Result<(), String>;
	fn chores(&self);
}

impl Windows for Blackhole {
	// Run a PowerShell script
	// TODO: Replace these with winapi calls when the relevant functions are binded
	fn powershell(script: String) {
		match Command::new("powershell").args(&["-Command", &script])
		.creation_flags(CREATE_NO_WINDOW)
		.stderr(std::process::Stdio::piped())
		.stdout(std::process::Stdio::null())
		.stdin(std::process::Stdio::null())
		.output() {
			Err(err) => { Show::error(&format!("Error executing PowerShell script: {}", err)) },
			Ok(output) => {
				if output.stderr.len() > 0 {
					Show::error(&format!("PowerShell error:\n{}", String::from_utf8(output.stderr).unwrap_or_else(|_| String::from("String conversion error"))));
				}
			}
		};
	}

	// SHChangeNotify(SHCNE_UPDATEDIR, ...) tell a file or directory to update its icon
	fn change_notify(ptr: *const winapi::ctypes::c_void) {
		unsafe { SHChangeNotify(SHCNE_UPDATEDIR, SHCNF_PATHW, ptr, std::ptr::null_mut()); }
	}

	// Sets Windows file attributes
	fn set_file_attributes(path: &std::path::PathBuf, attr: winapi::shared::minwindef::DWORD) -> *const u16 {
		let path_utf16: Vec<u16> = path.as_os_str().encode_wide().chain(iter::once(0)).collect();
		let path_utf16_ptr = path_utf16.as_ptr();
		unsafe { winapi::um::fileapi::SetFileAttributesW(path_utf16_ptr, attr); }
		return path_utf16_ptr;
	}

	// Add $BLACKHOLE to the Quick Access tab
	fn quick_access(path: &std::path::PathBuf) {
		Blackhole::powershell(format!("$o = new-object -com shell.application\n$o.Namespace('{}').Self.InvokeVerb('pintohome')", path.display()));

		let quick_access_folder_path = match CString::new("shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}") {
			Ok(quick_access_folder_path) => quick_access_folder_path,
			Err(err) => { Show::error(&format!("String conversion error: {}", err)); return; }
		};
		Blackhole::change_notify(quick_access_folder_path.as_ptr() as *const _);
	}
	
	// Configures the $BLACKHOLE folder
	fn set_blackhole_attributes(path: &std::path::PathBuf) {
		// Set the file attributes of the blackhole directory itself
		let path_utf16_ptr = Blackhole::set_file_attributes(path, winapi::um::winnt::FILE_ATTRIBUTE_SYSTEM | winapi::um::winnt::FILE_ATTRIBUTE_READONLY);

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
		
		Blackhole::change_notify(path_utf16_ptr as *const _);
	}

	fn edit_context_menu_registry() {
		// Add blackhole.exe --send to the File Explorer context menu

		let exe_path = match std::env::current_exe() {
			Ok(exe_path) => exe_path,
			Err(error) => {
				Show::error(&format!("Error getting executable path: {}", error));
				return;
			}
		};

		let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

		for path in ["*\\shell\\Blackhole", "Folder\\shell\\blackhole"].iter() {
			match hkcr.create_subkey_with_flags(*path, KEY_WRITE) {
				Ok(blackhole_shell) => {
					match blackhole_shell.0.set_value("", &"Blackhole") {
						Ok(_) => (),
						Err(error) => {
							Show::error(&format!("Error setting registry key: {}", error));
							return;
						}
					}
			
					match blackhole_shell.0.set_value("Icon", &format!("\"{}\"", exe_path.display())) {
						Ok(_) => (),
						Err(error) => {
							Show::error(&format!("Error setting registry key: {}", error));
							return;
						}
					}
			
					match blackhole_shell.0.create_subkey_with_flags("command", KEY_WRITE) {
						Ok(command) => {
							match command.0.set_value("", &format!("{} --send \"%1\"", exe_path.display())) {
								Ok(_) => (),
								Err(error) => {
									Show::error(&format!("Error setting registry key: {}", error));
									return;
								}
							}
						},
						Err(error) => {
							Show::error(&format!("Error opening registry subkey: {}", error));
							return;
						}
					};
				},
				Err(error) => {
					Show::error(&format!("Error opening registry subkey: {}", error));
					return;
				}
			};
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

	fn move_items(from: &Path, to: &Path) -> Result<(), String> {
		let mut from_null_terminated = OsString::from(from);
		from_null_terminated.push("\0\0");
		let from_utf16: Vec<u16> = from_null_terminated.encode_wide().chain(iter::once(0)).collect();

		let mut to_null_terminated = OsString::from(to);
		to_null_terminated.push("\0\0");
		let to_utf16: Vec<u16> = to_null_terminated.encode_wide().chain(iter::once(0)).collect();

		let mut fileop = SHFILEOPSTRUCTW {
			hwnd: 0 as HWND,
			wFunc: FO_MOVE as UINT,
			pFrom: from_utf16.as_ptr() as PCZZWSTR,
			pTo: to_utf16.as_ptr() as PCZZWSTR,
			fFlags: FOF_ALLOWUNDO,
			fAnyOperationsAborted: 0,
			hNameMappings: std::ptr::null_mut(),
			lpszProgressTitle: std::ptr::null()
		};

		let result = unsafe { SHFileOperationW(&mut fileop as *mut SHFILEOPSTRUCTW) };
		match result {
			S_OK => Ok(()),
			_ => Err(result.to_string())
		}
	}

	fn chores(&self) {
		// If we're running Windows, add blackhole.exe --purge to the startup registry
		Blackhole::edit_startup_registry();

		// Add blackhole.exe --send to the context menu registry
		Blackhole::edit_context_menu_registry();
		
		// Set file/folder attributes
		Blackhole::set_blackhole_attributes(&self.path);

		// Add $BLACKHOLE to Quick Access links
		Blackhole::quick_access(&self.path);
	}
}