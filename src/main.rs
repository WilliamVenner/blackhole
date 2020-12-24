use std::env;

mod show;
use show::show::Show;

mod blackhole;
use blackhole::blackhole::Blackhole;

#[cfg(windows)] mod windows;
#[cfg(linux)] mod linux;

fn main() {
    let should_purge: bool = env::args_os().any(|arg| arg == "--purge");
    match Blackhole::new(should_purge) {
        Ok(blackhole) => { println!("Location: {}", blackhole.path.display()) },
        Err(error) => { Show::panic(&String::from(error)); }
    }
}
