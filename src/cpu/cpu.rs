use crate::{get_lsb, get_msb, get_u16, MemoryBus};

use super::{
    ArithmeticTarget, FlagsRegister, Instruction, LoadByteSource, LoadByteTarget, LoadType,
    PopTarget, PushSource, Registers,
};

pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
    is_halted: bool,
}

impl CPU {
    fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    fn read_next_word(&self) -> u16 {
        let least_significant_byte = self.bus.read_byte(self.pc + 1);
        let most_significant_byte = self.bus.read_byte(self.pc + 2);

        get_u16(most_significant_byte, least_significant_byte)
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

        if self.is_halted {
            return self.pc;
        }

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

                        get_u16(most_significant_byte, least_significant_byte)
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
            NOP => self.pc.wrapping_add(1),
            HALT => {
                self.is_halted = true;
                self.pc
            }
            _ => {
                todo!("Implement rest instructions")
            }
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);

        self.registers.f.zero = new_value == 0;
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

        get_u16(most_significant_byte, least_significant_byte)
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
