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

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

#[derive(Clone, Copy)]
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

impl From<FlagsRegister> for u8 {
    fn from(flags: FlagsRegister) -> u8 {
        (if flags.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flags.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flags.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flags.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl From<u8> for FlagsRegister {
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

impl JumpType {
    fn should_jump(&self, flags: &FlagsRegister) -> bool {
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

enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

enum LoadByteSource {
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

enum LoadTargetFromA {
    BC,
    DE,
    D16,
}

enum LoadAFromSource {
    BC,
    DE,
    D16,
}

enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    FromA(LoadTargetFromA),
    ToA(LoadAFromSource),
}

enum PushSource {
    BC,
    DE,
    HL,
    AF,
}

enum PopTarget {
    BC,
    DE,
    HL,
    AF,
}

enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpType),
    LD(LoadType),
    PUSH(PushSource),
    POP(PopTarget),
    CALL(JumpType),
    RET(JumpType),
}

impl Instruction {
    fn from_opcode(opcode: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_opcode_prefixed(opcode)
        } else {
            Instruction::from_opcode_not_prefixed(opcode)
        }
    }

    fn from_opcode_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x02 => None,
            _ => None,
        }
    }

    fn from_opcode_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x02 => None,
            _ => None,
        }
    }
}

struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}

impl CPU {
    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    fn read_next_word(&self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.pc + 1);
        let most_significant_byte = self.bus.read_byte(self.pc + 2);

        get_16b_n(most_significant_byte, least_significant_byte)
    }

    fn decode(&self, instruction_byte: u8, prefixed: bool) -> Instruction {
        if let Some(instruction) = Instruction::from_opcode(instruction_byte, prefixed) {
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

    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let instruction = self.decode(instruction_byte, prefixed);

        self.pc = self.execute(instruction);
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        use Instruction::*;
        use LoadType::*;

        match instruction {
            ADD(target) => match target {
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);

                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                _ => self.pc,
            },
            JP(jump_type) => self.jump(jump_type.should_jump(&self.registers.f)),
            LD(load_type) => match load_type {
                Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::B => self.registers.b,
                        LoadByteSource::C => self.registers.c,
                        LoadByteSource::D => self.registers.d,
                        LoadByteSource::E => self.registers.e,
                        LoadByteSource::H => self.registers.h,
                        LoadByteSource::L => self.registers.l,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                    };

                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::B => self.registers.b = source_value,
                        LoadByteTarget::C => self.registers.c = source_value,
                        LoadByteTarget::D => self.registers.d = source_value,
                        LoadByteTarget::E => self.registers.e = source_value,
                        LoadByteTarget::H => self.registers.h = source_value,
                        LoadByteTarget::L => self.registers.l = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                    }

                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                // FromA(target) => {
                //     match target {
                //         LoadTargetFromA::BC => self.registers.set_bc(self.registers.a as u16),
                //         LoadTargetFromA::DE => self.registers.set_de(self.registers.a as u16),
                //         LoadTargetFromA::D16 => {
                //             let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
                //             let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
                //             let address = get_16b_n(most_significant_byte, least_significant_byte);

                //             self.bus.write_byte(address, self.registers.a);
                //         }
                //     }

                //     match target {
                //         LoadTargetFromA::D16 => self.pc.wrapping_add(3),
                //         _ => self.pc.wrapping_add(1)
                //     }
                // },
                _ => {
                    todo!("Implement rest types")
                }
            },
            PUSH(source) => {
                let source_value = match source {
                    PushSource::AF => {
                        let least_significant_byte = u8::from(self.registers.f);
                        let most_significant_byte = self.registers.a;

                        get_16b_n(most_significant_byte, least_significant_byte)
                    }
                    PushSource::BC => self.registers.get_bc(),
                    PushSource::DE => self.registers.get_de(),
                    PushSource::HL => self.registers.get_hl(),
                };

                self.push(source_value);

                self.pc.wrapping_add(1)
            }
            POP(target) => {
                let value = self.pop();

                match target {
                    PopTarget::AF => {
                        let least_significant_byte = get_lsb(&value);
                        let most_significant_byte = get_msb(&value);

                        self.registers.a = most_significant_byte;
                        self.registers.f = FlagsRegister::from(least_significant_byte);
                    }
                    PopTarget::BC => self.registers.set_bc(value),
                    PopTarget::DE => self.registers.set_de(value),
                    PopTarget::HL => self.registers.set_hl(value),
                }

                self.pc.wrapping_add(1)
            }
            CALL(jump_type) => {
                let should_jump = jump_type.should_jump(&self.registers.f);

                self.call(should_jump)
            }
            RET(jump_type) => {
                let should_jump = jump_type.should_jump(&self.registers.f);

                self.return_(should_jump)
            }
            _ => {
                todo!("Implement rest instructions")
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
            self.read_next_word()
        } else {
            self.pc.wrapping_add(3)
        }
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, get_msb(&value));

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, get_lsb(&value));
    }

    fn pop(&mut self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let most_significant_byte = self.bus.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);

        get_16b_n(most_significant_byte, least_significant_byte)
    }

    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            self.read_next_word()
        } else {
            next_pc
        }
    }

    fn return_(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }
}

fn get_msb(value: &u16) -> u8 {
    ((value & 0xFF00) >> 8) as u8
}

fn get_lsb(value: &u16) -> u8 {
    (value & 0xFF) as u8
}

fn get_16b_n(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

fn main() {
    println!("Game Boy!");
}
