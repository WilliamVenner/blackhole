#![cfg_attr(feature="gui", windows_subsystem = "windows")]

use std::{env::{self, ArgsOs}, iter::Peekable};

mod show;
use show::show::Show;

mod blackhole;
use blackhole::blackhole::Blackhole;

#[cfg(target_os="windows")] mod windows;
#[cfg(target_os="linux")]   mod linux;
#[cfg(target_os="macos")]   mod macos;

#[cfg(target_os="windows")] use crate::windows::Windows;
#[cfg(target_os="linux")]   use crate::linux::Linux;
#[cfg(target_os="macos")]   use crate::macos::MacOS;

#[cfg(feature="gui")]
use opener::open;

enum Operation {
	INITIALIZE,
	PURGE,
	SEND
}

fn get_operation() -> Result<(Operation, Peekable<ArgsOs>), String> {
	let mut args = env::args_os().peekable();

	// Consume the first argument, which is always the program's location
	args.nth(0);

	match args.nth(0) {
	    Some(command) => {
			match command.to_str().unwrap_or_default() {
				"--purge" => Ok((Operation::PURGE, args)),
		
				"--send" => {
					match args.peek() {
						Some(_) => Ok((Operation::SEND, args)),
						None => Err(String::from("--send argument present, but no file path was provided!"))
					}
				},

				_ => Ok((Operation::INITIALIZE, args))
			}
		},
	    None => Ok((Operation::INITIALIZE, args))
	}
}

fn main() {
	match get_operation() {
		Ok((operation, mut args)) => {
			match Blackhole::new() {
				Ok(blackhole) => {		
					match operation {
						Operation::SEND => { blackhole.send(args.nth(0).unwrap()); return }

						Operation::PURGE => blackhole.purge(),
		
						Operation::INITIALIZE => {
							#[cfg(feature="gui")]
							open(&blackhole.path).ok();
		
							Show::msg(&String::from("Blackhole directory initialized!"));
						},
					}
					
					// Run any chores
					#[cfg(any(target_os="windows", target_os="linux", target_os="macos"))]
					blackhole.chores();
				},
				Err(error) => { Show::panic(&String::from(error)); }
			}
		}

		Err(err) => Show::panic(&err)
	}
}