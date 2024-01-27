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
	}
}
