use std::{path::PathBuf, sync::atomic::AtomicPtr};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FolderLocation {
	HomeDir(
		#[serde(skip_serializing)]
		#[serde(deserialize_with = "home_dir")]
		PathBuf
	),
	Custom(PathBuf)
}
impl std::ops::Deref for FolderLocation {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target {
		match self {
			FolderLocation::HomeDir(path) => path,
			FolderLocation::Custom(path) => path,
		}
	}
}
impl Default for FolderLocation {
	fn default() -> Self {
		dirs::home_dir().map(FolderLocation::HomeDir).unwrap_or_else(|| FolderLocation::Custom(std::env::temp_dir()))
	}
}

static SETTINGS: AtomicPtr<Settings> = AtomicPtr::new(std::ptr::null_mut());

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Settings {
	pub folder_location: FolderLocation
}
impl Settings {
	fn path() -> PathBuf {
		let mut conf = dirs::config_dir().expect("This OS doesn't have a configuration directory to store our settings in!");
		conf.push("BLACKHOLE/settings.json");
		conf
	}

	fn read() -> Result<Settings, Box<dyn std::error::Error>> {
		let path = Settings::path();
		if !path.is_file() {
			let settings = Settings::default();
			settings.save().ok();
			Ok(settings)
		} else {
			let f = std::fs::File::open(path)?;
			Ok(serde_json::from_reader(f)?)
		}
	}

	pub fn load() {
		let settings = Settings::read().unwrap_or_default();
		let settings = Box::into_raw(Box::new(settings));
		let prev = SETTINGS.swap(settings, std::sync::atomic::Ordering::AcqRel);
		if prev != std::ptr::null_mut() {
			unsafe { Box::from_raw(prev) }; // drop it
		}
		println!("Settings: {:#?}", unsafe { &*settings });
	}

	pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
		let json = serde_json::to_string_pretty(self)?;
		std::fs::write(Settings::path(), json)?;
		Ok(())
	}
	
	pub fn modify<F: Fn(Settings) -> Settings>(&self, f: F) -> Result<(), Box<dyn std::error::Error>> {
		let new = f(self.clone());
		let new = Box::into_raw(Box::new(new));
		let prev = SETTINGS.swap(new, std::sync::atomic::Ordering::AcqRel);
		if prev != std::ptr::null_mut() {
			unsafe { Box::from_raw(prev) }; // drop it
		}
		unsafe { &*new }.save()
	}

	pub fn get() -> &'static Settings {
		let ptr: *mut Settings = SETTINGS.load(std::sync::atomic::Ordering::Relaxed);
		assert_ne!(ptr, std::ptr::null_mut(), "Settings have not been loaded yet");
		unsafe { &*ptr }
	}
}

macro_rules! setting {
	($ident:ident) => {
		crate::settings::Settings::get().$ident
	};
}