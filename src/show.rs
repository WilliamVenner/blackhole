pub mod show {
	pub struct Show();
	
	#[cfg(not(feature="gui"))]
	impl Show {
		pub fn panic(err: &String) {
			panic!("Blackhole PANIC: {}", err);
		}

		pub fn error(err: &String) {
			println!("Blackhole ERROR: {}", err);
		}

		pub fn msg(err: &String) {
			println!("Blackhole INFO: {}", err);
		}
	}

	#[cfg(feature="gui")] extern crate msgbox;
	#[cfg(feature="gui")] use msgbox::IconType;
	#[cfg(feature="gui")]
	impl Show {
		pub fn panic(err: &String) {
			let err = format!("{}\n\n{:#?}", err, backtrace::Backtrace::new());
			if msgbox::create("Blackhole PANIC", &err, IconType::Error).is_err() {
				panic!("Blackhole PANIC: {}", err);
			}
		}
	
		pub fn error(err: &String) {
			let err = format!("{}\n\n{:#?}", err, backtrace::Backtrace::new());
			if msgbox::create("Blackhole ERROR", &err, IconType::Error).is_err() {
				eprintln!("Blackhole ERROR: {}", err);
			}
		}
	
		pub fn msg(err: &String) {
			if msgbox::create("Blackhole", err, IconType::Info).is_err() {
				println!("Blackhole INFO: {}", err);
			}
		}
	}
}