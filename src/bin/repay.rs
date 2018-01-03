extern crate repay;

use std::io;
use std::io::prelude::*;

use repay::run;

fn main() {
    let stdin = io::stdin();
    for debt in run(stdin.lock().lines().map(|s| s.unwrap())) {
        println!("{}", debt);
    }
}
