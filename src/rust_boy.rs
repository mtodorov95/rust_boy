#[derive(Default, Clone, Copy)]
struct FlagsRegister {
    /// Set to true if the result of the operation is 0
    zero: bool,
    /// Set to true if the operation is a subtraction
    subtract: bool,
    /// Set to true if there is an overflow from the lower four bits to the
    /// upper four bits after the operation:
    /// 0b10001111 -> 0b10010000
    half_carry: bool,
    /// Set to true if an overflow did occur
    carry: bool,
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> 7) & 0b1) != 0;
        let subtract = ((byte >> 6) & 0b1) != 0;
        let half_carry = ((byte >> 5) & 0b1) != 0;
        let carry = ((byte >> 4) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> Self {
        (if flag.zero { 1 } else { 0 }) << 7
            | (if flag.subtract { 1 } else { 0 }) << 6
            | (if flag.half_carry { 1 } else { 0 }) << 5
            | (if flag.carry { 1 } else { 0 }) << 4
    }
}

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
}

impl Registers {
    fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister::default(),
            h: 0,
            l: 0,
        }
    }

    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (u8::from(self.f) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xFF) as u8);
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

struct CPU {
    registers: Registers,
    pc: u16,
    memory: [u8; 0xFFFF],
}

impl CPU {
    fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0x100,
            memory: [0; 0xFFFF],
        }
    }

    fn cycle(&mut self) {
        let mut byte = self.read_byte(self.pc);
        let prefixed = byte == 0xCB;

        if prefixed {
            byte = self.read_byte(self.pc+1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(byte, prefixed) {
            self.execute(instruction)
        } else {
            panic!("Unknown instruction: 0x{}{:x} found at: 0x{:x}", if prefixed {"cb"} else {""},byte, self.pc)
        }

        self.pc = next_pc;
    }

    fn read_next_byte(&self) -> u8 {
        self.read_byte(self.pc+1)
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
            Instruction::LD(load_type) => {
                match load_type {
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
                            LoadByteTarget::B => self.registers.b= value,
                            LoadByteTarget::C => self.registers.c= value,
                            LoadByteTarget::D => self.registers.d= value,
                            LoadByteTarget::E => self.registers.e= value,
                            LoadByteTarget::H => self.registers.h= value,
                            LoadByteTarget::L => self.registers.l= value,
                            LoadByteTarget::HL => self.write_byte(self.registers.get_hl(), value),
                        };

                        match source {
                            LoadByteSource::D8 => self.pc.wrapping_add(2),
                            _ => self.pc.wrapping_add(1)
                        }
                    },
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
                    JumpCondition::Always => true
               }; 
               self.jump(should_jump)
            }
            _ => {
                self.pc
            }
        }
    }

    /// Returns the next value of the program counter
    fn jump(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            let lower = self.read_byte(self.pc+1) as u16;
            let upper = self.read_byte(self.pc+2) as u16;
            (upper << 8) | lower
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

enum JumpCondition {
    Always
}

enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpCondition),
    LD(LoadType),
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Self> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_non_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Self> {
        match byte {
            // TODO: Add 
            _ => None
        }
    }

    fn from_byte_non_prefixed(byte: u8) -> Option<Self> {
        match byte {
            // TODO: Add the rest
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0xC3 => Some(Instruction::JP(JumpCondition::Always)),
            _ => None
        }
    }
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
    D8,
}

enum LoadType {
    //TODO: Add the rest of the load types
    Byte(LoadByteSource, LoadByteTarget),
}
