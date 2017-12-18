use std::io::prelude::*;
use std::io::stdin;
use std::convert::TryFrom;
use std::mem::transmute;

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum OpCodes {
	HALT = 0,
	SET,
	PUSH, POP,
	EQ,	GT,
	JMP, JT, JF,
	ADD, MULT, MOD,
	AND, OR, NOT,
	RMEM, WMEM, 
	CALL, RET,
	OUT, IN,
	NOOP
}

impl TryFrom<u16> for OpCodes {
	type Error = &'static str;
	fn try_from(op: u16) -> Result<Self, Self::Error> {
		if op > unsafe { transmute(OpCodes::NOOP) } { return Err("Invalid opcode."); }
		return unsafe { Ok(transmute(op)) };
	}
}

#[derive(Debug, Copy, Clone)]
enum Argument {
	LiteralValue(u16),
	Register(usize)
}
use self::Argument::*;

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

pub struct VM {
	data: Vec<u16>,
	registers: [u16; 8],
	stack: Vec<u16>,
	isp: usize,
}

impl VM {
	pub fn new(data: Vec<u16>) -> VM {
		VM { data, registers: [0; 8], stack: Vec::new(), isp: 0 }
	}
	
	pub fn run(mut self) {
		loop {
			self.step().unwrap();
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
	
	fn step(&mut self) -> Result<(), &'static str>{
		self.isp += 1;

		match TryFrom::<u16>::try_from(self.data[self.isp - 1])? {
			OpCodes::HALT => return Ok(()),
			OpCodes::SET => {
				let r = self.argument();
				let b = self.argument();
				
				self.registers[r.get_register_slot()?] = b.get_contained_value(&self);
			}
			OpCodes::PUSH => {
				let _value = self.argument().get_contained_value(&self);				
				self.stack.push(_value);
			}
			OpCodes::POP => {
				let r = self.argument();
				
				self.registers[r.get_register_slot()?] = self.stack.pop().unwrap();
			}
			OpCodes::EQ => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();

				self.registers[r.get_register_slot()?] = (b.get_contained_value(&self) == c.get_contained_value(&self)) as u16;					
			}
			OpCodes::GT => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();

				self.registers[r.get_register_slot()?] = (b.get_contained_value(&self) > c.get_contained_value(&self)) as u16;
			}
			OpCodes::JMP => {
				self.isp = self.argument().get_contained_value(&self) as usize;				
			}
			OpCodes::JT => {
				let cond = self.argument();
				let jmp = self.argument();
				
				if cond.get_contained_value(&self) != 0 {
					self.isp = jmp.get_contained_value(&self) as usize;
				}
			}
			OpCodes::JF => {
				let cond = self.argument();
				let jmp = self.argument();
				
				if cond.get_contained_value(&self) == 0 {
					self.isp = jmp.get_contained_value(&self) as usize;
				}
			}
			OpCodes::ADD => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();
				
				self.registers[r.get_register_slot().unwrap()] = {
					(b.get_contained_value(self) + c.get_contained_value(self)) % 32768
				};
			}
			OpCodes::MULT => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();
				
				self.registers[r.get_register_slot().unwrap()] = {
					(b.get_contained_value(self).wrapping_mul(c.get_contained_value(self))) % 32768
				};
			}
			OpCodes::MOD => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();
				
				self.registers[r.get_register_slot().unwrap()] = {
					(b.get_contained_value(self) % c.get_contained_value(self))
				};
			}
			OpCodes::AND => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();
				
				self.registers[r.get_register_slot()?] = {
					b.get_contained_value(&self) & c.get_contained_value(self)
				}
			}
			OpCodes::OR => {
				let r = self.argument();
				let b = self.argument();
				let c = self.argument();
				
				self.registers[r.get_register_slot()?] = {
					b.get_contained_value(&self) | c.get_contained_value(self)
				}	
			}
			OpCodes::NOT => {
				let r = self.argument();
				let b = self.argument();
				
				self.registers[r.get_register_slot()?] = !b.get_contained_value(self) & 0x7FFF;
			}
		 	OpCodes::RMEM => {
				let r = self.argument();
				let b = self.argument();
				
				self.registers[r.get_register_slot()?] = self.data[b.get_contained_value(self) as usize];
			}
			OpCodes::WMEM => {
				let a = self.argument();
				let b = self.argument();
				
				let index = a.get_contained_value(self) as usize;
				self.data[index] = b.get_contained_value(self);
			}
			OpCodes::CALL => {
				let isp = self.argument();
				
				self.stack.push(self.isp as u16);
				self.isp = isp.get_contained_value(self) as usize;
			}
			OpCodes::RET => {				
				self.isp = self.stack.pop().unwrap() as usize;
			}
			OpCodes::OUT => {
				print!("{}", self.argument().get_contained_value(&self) as u8 as char);
			}
			OpCodes::IN => {
				let a = self.argument();
				let input = stdin().chars().next().unwrap();
				
				self.registers[a.get_register_slot()?] = input.unwrap() as u8 as u16;
			}
			OpCodes::NOOP => {	}	
		};
		
		Ok(())
	}
}