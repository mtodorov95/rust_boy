use instruction::{
    ArithmeticTarget, Instruction, JumpCondition, LoadByteSource, LoadByteTarget, LoadType,
    StackTarget,
};
use registers::Registers;

mod instruction;
mod registers;

pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: [u8; 0xFFFF],
    is_halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0x100,
            sp: 0x00,
            memory: [0; 0xFFFF],
            is_halted: false,
        }
    }

    fn cycle(&mut self) {
        let mut byte = self.read_byte(self.pc);
        let prefixed = byte == 0xCB;

        if prefixed {
            byte = self.read_byte(self.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(byte, prefixed) {
            self.execute(instruction)
        } else {
            panic!(
                "Unknown instruction: 0x{}{:x} found at: 0x{:x}",
                if prefixed { "cb" } else { "" },
                byte,
                self.pc
            )
        };

        if (!self.is_halted) {
            self.pc = next_pc;
        }
    }

    fn read_next_word(&self) -> u16 {
        let lo = self.read_byte(self.pc + 1) as u16;
        let hi = self.read_byte(self.pc + 2) as u16;
        (hi << 8) | lo
    }

    fn read_next_byte(&self) -> u8 {
        self.read_byte(self.pc + 1)
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    /// Executes the current instruction and returns the next value of the
    /// program counter
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::NOP => self.pc.wrapping_add(1),
            Instruction::HALT => {
                self.is_halted = true;
                self.pc.wrapping_add(1)
            }
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(source, target) => {
                    let value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::B => self.registers.b,
                        LoadByteSource::C => self.registers.c,
                        LoadByteSource::D => self.registers.d,
                        LoadByteSource::E => self.registers.e,
                        LoadByteSource::H => self.registers.h,
                        LoadByteSource::L => self.registers.l,
                        LoadByteSource::HL => self.read_byte(self.registers.get_hl()),
                        LoadByteSource::D8 => self.read_next_byte(),
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = value,
                        LoadByteTarget::B => self.registers.b = value,
                        LoadByteTarget::C => self.registers.c = value,
                        LoadByteTarget::D => self.registers.d = value,
                        LoadByteTarget::E => self.registers.e = value,
                        LoadByteTarget::H => self.registers.h = value,
                        LoadByteTarget::L => self.registers.l = value,
                        LoadByteTarget::HL => self.write_byte(self.registers.get_hl(), value),
                    };

                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
            },
            Instruction::ADD(target) => match target {
                ArithmeticTarget::A => todo!(),
                ArithmeticTarget::B => todo!(),
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::D => todo!(),
                ArithmeticTarget::E => todo!(),
                ArithmeticTarget::H => todo!(),
                ArithmeticTarget::L => todo!(),
            },
            Instruction::JP(condition) => {
                let should_jump = match condition {
                    JumpCondition::Always => true,
                    _ => panic!("Add all jump conditions"),
                };
                self.jump(should_jump)
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                    StackTarget::AF => self.registers.get_af(),
                };
                self.push(value);
                self.pc.wrapping_add(1)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::BC => self.registers.set_bc(result),
                    StackTarget::DE => self.registers.set_de(result),
                    StackTarget::HL => self.registers.set_hl(result),
                    StackTarget::AF => self.registers.set_af(result),
                }
                self.pc.wrapping_add(1)
            }
            Instruction::CALL(condition) => {
                let should_jump = match condition {
                    JumpCondition::NotZero => !self.registers.f.zero,
                    _ => panic!("Add all jump conditions"),
                };
                self.call(should_jump)
            }
            Instruction::RET(condition) => {
                let should_jump = match condition {
                    JumpCondition::NotZero => !self.registers.f.zero,
                    _ => panic!("Add all jump conditions"),
                };
                self.ret(should_jump)
            }
            _ => self.pc,
        }
    }

    fn call(&mut self, should_jump: bool) -> u16 {
        // The next address if we didn't have to jump.
        // Skips the instruction and the 2 bytes for the jump address
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn ret(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, (value & 0x00FF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lo = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (hi << 8) | lo
    }

    /// Returns the next value of the program counter
    fn jump(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.read_next_word()
        } else {
            // Just skip the 3 bytes of the jump instruction
            self.pc.wrapping_add(3)
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = overflow;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }
}
