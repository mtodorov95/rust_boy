pub enum JumpCondition {
    Always,
    NotZero,
    NotCarry,
    Zero,
    Carry,
}

pub enum Instruction {
    NOP,
    HALT,
    ADD(ArithmeticTarget),
    JP(JumpCondition),
    LD(LoadType),
    PUSH(StackTarget),
    POP(StackTarget),
    CALL(JumpCondition),
    RET(JumpCondition),
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Self> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_non_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Self> {
        match byte {
            // TODO: Add
            _ => None,
        }
    }

    fn from_byte_non_prefixed(byte: u8) -> Option<Self> {
        match byte {
            // TODO: Add the rest
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0xC3 => Some(Instruction::JP(JumpCondition::Always)),
            _ => None,
        }
    }
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

pub enum LoadByteSource {
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

pub enum LoadType {
    //TODO: Add the rest of the load types
    Byte(LoadByteSource, LoadByteTarget),
}

pub enum StackTarget {
    BC,
    DE,
    HL,
    AF,
}
