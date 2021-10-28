#[macro_use]
extern crate serde;

#[macro_use]
mod settings;
mod sys;
mod ops;

fn init_state() {
	settings::Settings::load();
}

fn main() {
	let op = match std::env::args().skip(1).next().as_deref() {
		Some("init") => ops::init,
		Some("purge") => ops::purge,
		Some(_) | None => return eprintln!("No operation specified"),
	};

	init_state();
	op();
}
