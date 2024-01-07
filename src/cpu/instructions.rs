use super::FlagsRegister;

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

pub enum JumpType {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpType {
    pub fn should_jump(&self, flags: &FlagsRegister) -> bool {
        use JumpType::*;

        match self {
            NotZero => !flags.zero,
            Zero => flags.zero,
            NotCarry => !flags.carry,
            Carry => flags.carry,
            Always => true,
        }
    }
}

pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

pub enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLI,
}

pub enum LoadTargetFromA {
    BC,
    DE,
    D16,
}

pub enum LoadAFromSource {
    BC,
    DE,
    D16,
}

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    FromA(LoadTargetFromA),
    ToA(LoadAFromSource),
}

pub enum PushSource {
    BC,
    DE,
    HL,
    AF,
}

pub enum PopTarget {
    BC,
    DE,
    HL,
    AF,
}

pub enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpType),
    LD(LoadType),
    PUSH(PushSource),
    POP(PopTarget),
    CALL(JumpType),
    RET(JumpType),
    NOP,
    HALT,
}

impl Instruction {
    pub fn from_opcode(opcode: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_opcode_prefixed(opcode)
        } else {
            Instruction::from_opcode_not_prefixed(opcode)
        }
    }

    pub fn from_opcode_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x02 => None,
            _ => None,
        }
    }

    pub fn from_opcode_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x02 => None,
            _ => None,
        }
    }
}
