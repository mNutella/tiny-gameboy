// TODO:
// CPU(ALU, Registers, Instruction Address, Instruction Counter)
// Memory(RAM)
// Memory(ROM)
// Graphic Memory(Matrix)
// I/O(Screen, Controller)

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
    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
}

struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flags: FlagsRegister) -> u8 {
        (if flags.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flags.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flags.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flags.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> FlagsRegister {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpType),
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

enum JumpType {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x02 => None,
            _ => None,
        }
    }

    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x02 => None,
            _ => None,
        }
    }
}

struct CPU {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn fetch(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl CPU {
    fn step(&mut self) {
        let mut instruction_byte = self.bus.fetch(self.pc);
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            instruction_byte = self.bus.fetch(self.pc.wrapping_add(1));
        }

        let instruction = self.decode(instruction_byte, prefixed);

        self.pc = self.execute(instruction);
    }

    fn decode(&self, instruction_byte: u8, prefixed: bool) -> Instruction {
        if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
            instruction
        } else {
            let description = format!(
                "0x{}{:x}",
                if prefixed { "cb" } else { "" },
                instruction_byte
            );
            panic!("Unkown instruction found for: {}", description);
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        use ArithmeticTarget::*;
        use Instruction::*;
        use JumpType::*;

        match instruction {
            ADD(target) => match target {
                C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);

                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                _ => self.pc,
            },
            JP(jump_type) => {
                let jump_condition = match jump_type {
                    NotZero => !self.registers.f.zero,
                    Zero => self.registers.f.zero,
                    NotCarry => !self.registers.f.carry,
                    Carry => self.registers.f.carry,
                    Always => true,
                };

                self.jump(jump_condition)
            }
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

        new_value
    }

    fn jump(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            let least_significant_byte = self.bus.fetch(self.pc + 1) as u16;
            let most_significant_byte = self.bus.fetch(self.pc + 2) as u16;

            (most_significant_byte << 8) | least_significant_byte
        } else {
            self.pc.wrapping_add(3)
        }
    }
}

fn main() {
    println!("Game Boy!");
}
