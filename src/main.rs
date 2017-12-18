#![feature(exclusive_range_pattern)]
#![feature(io)]
#![feature(try_from)]

mod vm;
use vm::VM;

use std::fs::{File};
use std::io::prelude::*;
use std::mem;

fn main() {	
	fn load_program() -> Vec<u16> {
		let mut file = File::open("challenge.bin").unwrap();
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();
		
		unsafe {
			let new_data = Vec::<u16>::from_raw_parts(data.as_mut_ptr() as *mut u16, data.len() / 2, data.len() / 2);
			mem::forget(data);
			return new_data;
		}
	}
	
	let data = load_program();
	let vm = VM::new(data);
	vm.run();
	
	println!("Execution terminated.");
}

