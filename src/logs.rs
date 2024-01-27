use log::Log;
use std::{fs::File, io::Write};

use crate::Operation;

struct Logger(Option<File>);
impl Logger {
	fn new(op: Operation) -> Self {
		Self(
			match File::create(std::env::temp_dir().join(match op {
				Operation::Initialize => "BLACKHOLE.log",
				Operation::Purge => "BLACKHOLE_PURGE.log",
				#[cfg(windows)]
				Operation::SendTo => "BLACKHOLE_SENDTO.log",
			})) {
				Ok(f) => Some(f),
				Err(err) => {
					eprintln!("Failed to create log file: {err:?}");
					None
				}
			},
		)
	}
}
impl Log for Logger {
	#[inline]
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		metadata.level() <= log::Level::Info
	}

	fn flush(&self) {
		let Some(mut f) = self.0.as_ref() else { return };
		(&mut f).flush().ok();
	}

	fn log(&self, record: &log::Record) {
		let log = format!("[{}] [{}]: {}", chrono::Local::now(), record.level(), record.args());

		if record.level() == log::Level::Error {
			eprintln!("{}", log);
		} else {
			println!("{}", log);
		}

		let Some(mut f) = self.0.as_ref() else { return };
		let f = &mut f;
		writeln!(f, "{}", log).ok();
	}
}

pub fn init(op: Operation) {
	log::set_boxed_logger(Box::new(Logger::new(op))).ok();
	log::set_max_level(log::LevelFilter::Info);
}
