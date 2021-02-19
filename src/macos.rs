embed_plist::embed_launchd_plist!("../src/assets/launchd.plist");

use cocoa::{self, appkit::NSImage, base::nil, foundation::NSString};
use objc::{self, msg_send, sel, sel_impl, class, runtime::{YES, Object}};

use std::{fs, path::PathBuf, str};

use crate::Blackhole;
use crate::Show;

enum IconSetError {
	NSImage,
	SetIcon
}

pub trait MacOS {
	fn launchd();
	fn set_blackhole_icon(path: &PathBuf);
	fn chores(&self);
}

impl MacOS for Blackhole {
	fn launchd() {
		let mut plist_path = match dirs::home_dir() {
			Some(plist_path) => plist_path,
			None => { Show::panic(&String::from("Could not find a home directory!")); return; },
		};

		let exe_path = match std::env::current_exe() {
			Ok(exe_path) => {
				match exe_path.to_str() {
					Some(exe_path) => exe_path.to_owned(),
					None => { Show::error(&String::from("Failed to convert executable path to str")); return; }
				}
			},
			Err(error) => { Show::error(&format!("Error getting executable path: {}", error)); return; }
		};
		
		plist_path.push("Library/LaunchAgents/com.venner.blackhole.plist");
		
		let embedded_plist_bytes = embed_plist::get_launchd_plist();
		let embedded_plist = match str::from_utf8(embedded_plist_bytes) {
			Ok(embedded_plist) => embedded_plist,
			Err(error) => { Show::error(&format!("Error converting launchd.plist to UTF-8: {}", error)); return; }
		};

		match fs::write(plist_path, embedded_plist.replacen("$BLACKHOLE_EXE_PATH", &exe_path, 1)) {
			Ok(_) => { println!("Added to launchd jobs successfully"); }
			Err(error) => { Show::error(&format!("Error writing to launchd job plist file: {}", error)); }
		};
	}

	fn set_blackhole_icon(path: &PathBuf) {
		let icon_path: String = match std::env::current_exe() {
			Ok(mut icon_path) => {
				icon_path.pop(); icon_path.pop(); icon_path.push("Resources/blackhole.icns");
				match icon_path.to_str() {
					Some(icon_path) => icon_path.to_owned(),
					None => { Show::error(&String::from("Failed to convert icon path to str")); return; }
				}
			},
			Err(error) => { Show::error(&format!("Error getting executable path: {}", error)); return; }
		};

		let blackhole_path: &str = match path.to_str() {
			Some(blackhole_path) => blackhole_path,
			None => { Show::error(&String::from("Failed to convert blackhole directory path to str")); return; }
		};

		let mut icon_fail_reason: Option<IconSetError> = None;
		unsafe {
			let autorelease_pool: *mut Object = msg_send![class!(NSAutoreleasePool), new];
			
			let ns_icon_path: *mut Object = NSString::alloc(nil).init_str(icon_path.as_str());
			let icon: *mut Object = NSImage::alloc(nil).initByReferencingFile_(ns_icon_path);
			if icon.isValid() == YES {
				let shared_workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
				
				let ns_blackhole_path: *mut Object = NSString::alloc(nil).init_str(blackhole_path);
				let success: *mut Object = msg_send![shared_workspace, setIcon:icon forFile:ns_blackhole_path options:0];
				if success as i8 != YES {
					icon_fail_reason = Some(IconSetError::SetIcon);
				}
			} else {
				icon_fail_reason = Some(IconSetError::NSImage);
			}

			let _: () = msg_send![autorelease_pool, release];
		}

		if icon_fail_reason.is_none() {
			println!("Set Blackhole folder icon successfully!");
			return;
		}

		Show::error(&format!("Failed to set Blackhole directory icon! ({})", &String::from(match icon_fail_reason.unwrap() {
			IconSetError::NSImage => "Failed to init NSImage",
			IconSetError::SetIcon => "setIcon failed"
		})));
	}

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

	fn chores(&self) {
		// Set $BLACKHOLE folder icon
		Blackhole::set_blackhole_icon(&self.path);

		// Add --purge to launchd jobs
		Blackhole::launchd();
	}
}