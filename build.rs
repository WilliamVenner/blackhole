#[macro_use]
extern crate build_cfg;

#[build_cfg_main]
fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=assets/blackhole.ico");

	if build_cfg!(windows) {
		winres::WindowsResource::new()
			.set_icon("assets/blackhole.ico")
			.compile()
			.expect("Failed to set icon");
	} else if build_cfg!(target_os = "macos") {
		println!("cargo:rerun-if-changed=src/os/macos.m");

		cc::Build::new()
			.file("src/os/macos.m")
			.cargo_metadata(true)
			.flag("-fmodules")
			.compile("blackhole_macos_objc");
	}
}
