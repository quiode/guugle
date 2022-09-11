use std::{thread, time::Duration};

pub fn run() {
    print!("Loading");
    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        print!(".");
    }
    todo!()
}
