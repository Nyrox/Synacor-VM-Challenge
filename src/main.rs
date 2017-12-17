#![feature(exclusive_range_pattern)]
#![feature(io)]

use std::fs::{File};
use std::io::prelude::*;


fn main() {	
	fn load_program() -> Vec<u16> {
		let mut file = File::open("challenge.bin").unwrap();
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();
		
		unsafe {
			let new_data = Vec::<u16>::from_raw_parts(data.as_mut_ptr() as *mut u16, data.len() / 2, data.len() / 2);
			::std::mem::forget(data);
			return new_data;
		}
	}
	
	let data = load_program();
	let mut vm = VM::new(data);
	vm.run();
	
	println!("Execution terminated.");
}



macro_rules! VM_LOG {
	($self: ident, $str: expr) => {{
		let _result = $str;
		$self.log(_result);
	}}
}

#[derive(Debug)]
enum Argument {
	LiteralValue(u16),
	Register(usize)
}
use Argument::*;

impl Argument {
	
	fn get_contained_value(&self, vm: &VM) -> u16 {
		match *self {
			LiteralValue(u) => u,
			Register(r) => vm.registers[r]
		}
	}
	
	fn get_register_slot(&self) -> Result<usize, &'static str> {
		match *self {
			LiteralValue(_) => Err("Tried to retrieve register slot on literal value."),
			Register(r) => Ok(r)
		}
	}
}

struct VM {
	data: Vec<u16>,
	isp: usize,
	registers: [u16; 8],
	stack: Vec<u16>,
	sp: usize,
	log: File
}

impl VM {
	fn new(data: Vec<u16>) -> VM {
		print!("\n\n");
		VM { data, isp: 0, registers: [0; 8], stack: Vec::new(), sp: 0, log: File::create("info.log").unwrap() }
	}
	
	fn log(&mut self, str: &str) {
		self.log.write(str.as_bytes());
	}
	
	fn run(&mut self) {
		
		loop {
			self.isp += 1;
			VM_LOG!(self, &format!("\nRegisters: {:?}", self.registers));
			VM_LOG!(self, &format!("\nExecuting instruction: {}", self.isp - 1));
			match self.data[self.isp - 1] {
				0 => return,
				1 => {
					let r = self.argument();
					let b = self.argument();
					
					VM_LOG!(self, &format!(": SET (a: {:?}, b: {:?})", r, b));
					
					self.registers[r.get_register_slot().unwrap()] = b.get_contained_value(&self);
				}
				2 => {
					let v = self.argument();
					
					VM_LOG!(self, &format!(": PUSH (a: {:?})", v));
					
					let v = v.get_contained_value(self);
					self.stack.push(v);
				}
				3 => {
					let r = self.argument();
					
					VM_LOG!(self, &format!(": POP (a: {:?})", r));
					
					self.registers[r.get_register_slot().unwrap()] = self.stack.pop().expect("Tried to pop of an empty stack");
				}
				4 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": EQ (a: {:?}, b: {:?}, c: {:?})", r, b, c));					
					
					self.registers[r.get_register_slot().unwrap()] = (b.get_contained_value(&self) == c.get_contained_value(&self)) as u16;					
				}
				5 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": GT (a: {:?}, b: {:?}, c: {:?})", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = (b.get_contained_value(&self) > c.get_contained_value(&self)) as u16;
				}
				6 => {
					let isp = self.argument();
					VM_LOG!(self, &format!(": JMP (a: {:?})", isp));
					self.isp = isp.get_contained_value(&self) as usize;
					continue;
				}
				7 => {
					let cond = self.argument();
					let jmp = self.argument();
					
					VM_LOG!(self, &format!(": JT (a: {:?}, b: {:?})", cond, jmp));
					
					if cond.get_contained_value(&self) != 0 {
						self.isp = jmp.get_contained_value(&self) as usize;
						continue;
					}
				}
				8 => {
					let cond = self.argument();
					let jmp = self.argument();
					
					VM_LOG!(self, &format!(": JF (a: {:?}, b: {:?})", cond, jmp));
					
					if cond.get_contained_value(&self) == 0 {
						self.isp = jmp.get_contained_value(&self) as usize;
						continue;
					}
				}
				9 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": ADD (a: {:?}, b: {:?}, c: {:?})", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = {
						(b.get_contained_value(self) + c.get_contained_value(self)) % 32768
					};
				}
				10 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": MUL (a: {:?}, b: {:?}, c: {:?})", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = {
						(b.get_contained_value(self).wrapping_mul(c.get_contained_value(self))) % 32768
					};
				}
				11 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": MOD (a: {:?}, b: {:?}, c: {:?})", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = {
						(b.get_contained_value(self) % c.get_contained_value(self))
					};
				}
				12 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": AND (a: {:?}, b: {:?}, c: {:?}))", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = {
						b.get_contained_value(&self) & c.get_contained_value(self)
					}
				}
				13 => {
					let r = self.argument();
					let b = self.argument();
					let c = self.argument();
					
					VM_LOG!(self, &format!(": OR (a: {:?}, b: {:?}, c: {:?}))", r, b, c));
					
					self.registers[r.get_register_slot().unwrap()] = {
						b.get_contained_value(&self) | c.get_contained_value(self)
					}	
				}
				14 => {
					let r = self.argument();
					let b = self.argument();
					
					VM_LOG!(self, &format!(": NOT (a: {:?}, b: {:?})", r, b));
					
					self.registers[r.get_register_slot().unwrap()] = !b.get_contained_value(self) & 0x7FFF;
				}
				15 => {
					let r = self.argument();
					let b = self.argument();
					
					VM_LOG!(self, &format!(": RMEM (a: {:?}, b: {:?})", r, b));
					
					self.registers[r.get_register_slot().unwrap()] = self.data[b.get_contained_value(self) as usize];
				}
				16 => {
					let a = self.argument();
					let b = self.argument();
					
					VM_LOG!(self, &format!(": WMEM (a: {:?}, b: {:?})", a, b));
					
					let index = a.get_contained_value(self) as usize;
					self.data[index] = b.get_contained_value(self);
				}
				17 => {
					let isp = self.argument();
					
					VM_LOG!(self, &format!(": CALL (a: {:?})", isp));
					
					self.stack.push(self.isp as u16);
					self.isp = isp.get_contained_value(self) as usize;
				}
				18 => {
					let isp = self.stack.pop().unwrap();
					
					VM_LOG!(self, &format!(": RET (a: {:?})", isp));
					
					self.isp = isp as usize;
				}
				19 => {
					let character = self.argument().get_contained_value(&self) as u8 as char;
					print!("{}", character);
					
					// VM_LOG!(self, &format!(": PRINT (a: {})", character));
				}
				20 => {
					let a = self.argument();
					let input = std::io::stdin().chars().next().unwrap().unwrap();
					
					VM_LOG!(self, &format!(": IN (a: {:?}, b: {:?})", a, input as u16));
					
					self.registers[a.get_register_slot().unwrap()] = input as u8 as u16;
				}
				21 => {
					VM_LOG!(self, ": NOOP");
				}
				i => {
					VM_LOG!(self, ": UNEXPECT");
					println!("Unexpected opcode: {}", i);
				}
			}
			
			
		}		
	}
	
	fn argument(&mut self) -> Argument {
		self.isp += 1;
		
		match self.data[self.isp - 1] {
			i @ 0..32768 => LiteralValue(i),
			i @ 32768..32776 => Register(i as usize - 32768),
			_ => panic!()
		}
	}
	
}