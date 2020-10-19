//!
//! Runs a given RISC-V program instruction by instruction.
//!
//! Implemented instructions can be found at [Instructions](./parser/enum.Instruction.html),
//! and you can find how they're simulated at [Simulator::run](struct.Simulator.html#method.run)
//!

use std::sync::{Arc, Mutex};

const DATA_SIZE: usize = 2048; // TODO: this
const MMIO_SIZE: usize = 0x201000;
const MMIO_START: usize = 0xff000000;

pub mod parser;
use parser::{Includable, MacroParseable, RISCVParser};

mod into_register;
use into_register::*;

use byteorder::{ByteOrder, LittleEndian};

pub struct Memory {
    pub mmio: Arc<Mutex<Vec<u8>>>,
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            mmio: Arc::new(Mutex::new(vec![0; MMIO_SIZE])),
            data: vec![0; DATA_SIZE],
        }
    }

    pub fn get_byte(&self, i: usize) -> u8 {
        if i >= MMIO_START {
            self.mmio.lock().unwrap()[i - MMIO_START]
        } else {
            self.data[i]
        }
    }

    pub fn set_byte(&mut self, i: usize, x: u8) {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            (*mmio)[i - MMIO_START] = x;
        } else {
            self.data[i] = x;
        }
    }

    pub fn get_half(&self, i: usize) -> u16 {
        if i >= MMIO_START {
            let mmio = self.mmio.lock().unwrap();
            LittleEndian::read_u16(&mmio[i - MMIO_START..])
        } else {
            LittleEndian::read_u16(&self.data[i..])
        }
    }

    pub fn set_half(&mut self, i: usize, x: u16) {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            LittleEndian::write_u16(&mut mmio[i - MMIO_START..], x);
        } else {
            LittleEndian::write_u16(&mut self.data[i..], x);
        }
    }

    pub fn get_word(&self, i: usize) -> u32 {
        if i >= MMIO_START {
            let mmio = self.mmio.lock().unwrap();
            LittleEndian::read_u32(&mmio[i - MMIO_START..])
        } else {
            LittleEndian::read_u32(&self.data[i..])
        }
    }

    pub fn set_word(&mut self, i: usize, x: u32) {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            LittleEndian::write_u32(&mut mmio[i - MMIO_START..], x);
        } else {
            LittleEndian::write_u32(&mut self.data[i..], x);
        }
    }
}

pub struct Simulator {
    registers: [u32; 32],
    _floats: [f32; 32],
    _status: Vec<u32>, // I'm not sure myself how many status register I'll use
    pc: usize,

    pub memory: Memory,
    pub code: Vec<parser::Instruction>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            _floats: [0.0; 32],
            _status: Vec::new(),
            pc: 0,
            memory: Memory::new(),
            code: Vec::new(),
        }
    }

    fn get_reg<T: FromRegister>(&self, i: u8) -> T {
        FromRegister::from(self.registers[i as usize])
    }

    fn set_reg<T: IntoRegister>(&mut self, i: u8, x: T) {
        // This could be made branchless by setting reg[i] = i == 0 ? 0 : x, but I'm not sure it's worth it
        if i != 0 {
            self.registers[i as usize] = x.into();
        }
    }

    pub fn load_from_file(mut self, path: String) -> Result<Self, parser::Error> {
        let pathbuf = std::path::PathBuf::from(&path);
        // TODO: some of this logic is duplicated from the Includer, try to dedup?
        let error = format!("Can't open file: <{:?}>", pathbuf.to_str());
        let parser::Parsed { code, data } = parser::file_lines(&path)
            .expect(&error)
            .parse_includes(pathbuf)
            .parse_macros()
            .parse_riscv(DATA_SIZE)?;

        self.code = code;
        self.memory.data = data;
        self.set_reg(2, self.memory.data.len() as u32 - 4); // set stack pointer
        Ok(self)
    }

    pub fn run(&mut self) {
        use parser::Instruction::*;

        let to_1 = |b| if b { 1 } else { 0 };

        macro_rules! branch {
            ($cond:expr, $pc:expr, $label:expr) => {
                if $cond {
                    $pc = $label;
                    continue;
                }
            };
        }

        loop {
            match self.code[self.pc / 4] {
                // Type R
                Add(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) + self.get_reg::<i32>(rs2))
                }
                Sub(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) - self.get_reg::<i32>(rs2))
                }
                Sll(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) << self.get_reg::<i32>(rs2))
                }
                Slt(rd, rs1, rs2) => self.set_reg(
                    rd,
                    to_1(self.get_reg::<i32>(rs1) < self.get_reg::<i32>(rs2)),
                ),
                Sltu(rd, rs1, rs2) => self.set_reg(
                    rd,
                    to_1(self.get_reg::<u32>(rs1) < self.get_reg::<u32>(rs2)),
                ),
                Xor(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) ^ self.get_reg::<u32>(rs2))
                }
                Srl(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) >> self.get_reg::<i32>(rs2))
                }
                Sra(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) >> self.get_reg::<i32>(rs2))
                }
                Or(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) | self.get_reg::<u32>(rs2))
                }
                And(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) & self.get_reg::<u32>(rs2))
                }
                Mul(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) * self.get_reg::<i32>(rs2))
                }
                Div(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) / self.get_reg::<i32>(rs2))
                }
                Divu(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) / self.get_reg::<u32>(rs2))
                }
                Rem(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) % self.get_reg::<i32>(rs2))
                }
                Remu(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<u32>(rs1) % self.get_reg::<u32>(rs2))
                }

                // Type I
                Ecall => self.ecall(),
                Addi(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) + (imm as i32)),
                Slli(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) << imm),
                Slti(rd, rs1, imm) => {
                    self.set_reg(rd, to_1(self.get_reg::<i32>(rs1) < (imm as i32)))
                }
                Sltiu(rd, rs1, imm) => self.set_reg(rd, to_1(self.get_reg::<u32>(rs1) < imm)),
                Xori(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) ^ imm),
                Srli(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) >> imm),
                Srai(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) >> imm),
                Ori(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) | imm),
                Andi(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) & imm),

                // Type I, loads from memory
                Lb(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_byte((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as u32,
                ),
                Lh(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_half((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as u32,
                ),
                Lw(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_word((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as u32,
                ),
                Lbu(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_byte((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as u8 as u32,
                ),
                Lhu(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_half((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as u16 as u32,
                ),

                // Type S
                Sb(rs2, imm, rs1) => self.memory.set_byte(
                    (self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize,
                    self.get_reg::<u8>(rs2),
                ),
                Sh(rs2, imm, rs1) => self.memory.set_half(
                    (self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize,
                    self.get_reg::<u16>(rs2),
                ),
                Sw(rs2, imm, rs1) => self.memory.set_word(
                    (self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize,
                    self.get_reg::<u32>(rs2),
                ),

                // Type SB + jumps
                Beq(rs1, rs2, label) => branch!(
                    self.get_reg::<i32>(rs1) == self.get_reg::<i32>(rs2),
                    self.pc,
                    label
                ),
                Bne(rs1, rs2, label) => branch!(
                    self.get_reg::<i32>(rs1) != self.get_reg::<i32>(rs2),
                    self.pc,
                    label
                ),
                Blt(rs1, rs2, label) => branch!(
                    self.get_reg::<i32>(rs1) < self.get_reg::<i32>(rs2),
                    self.pc,
                    label
                ),
                Bge(rs1, rs2, label) => branch!(
                    self.get_reg::<i32>(rs1) >= self.get_reg::<i32>(rs2),
                    self.pc,
                    label
                ),
                Bltu(rs1, rs2, label) => branch!(
                    self.get_reg::<u32>(rs1) < self.get_reg::<u32>(rs2),
                    self.pc,
                    label
                ),
                Bgeu(rs1, rs2, label) => branch!(
                    self.get_reg::<u32>(rs1) >= self.get_reg::<u32>(rs2),
                    self.pc,
                    label
                ),
                Jalr(rd, rs1, imm) => {
                    // This produces a weird result for `jalr s0 s0 label`. s0 is set to pc+4 before the jump occurs
                    // so it works as a nop. Maybe this is correct, maybe it's not, but I'll copy the behavior seen in
                    // RARS to be consistent.
                    self.set_reg(rd, (self.pc + 4) as u32);
                    self.pc = (self.get_reg::<i32>(rs1) + (imm as i32)) as usize & !1;
                    continue;
                }
                Jal(rd, label) => {
                    self.set_reg(rd, (self.pc + 4) as u32);
                    self.pc = label;
                    continue;
                }

                // Pseudoinstructions
                Li(rd, imm) => self.set_reg(rd, imm),
                Mv(rd, rs1) => self.registers[rd as usize] = self.registers[rs1 as usize],
                Ret => {
                    self.pc = self.registers[1] as usize;
                    continue;
                }
            }

            self.pc += 4;
        }
    }

    fn ecall(&mut self) {
        // 17 = a7
        match self.get_reg::<i32>(17) {
            10 => std::process::exit(0), // exit
            1 => {
                // print int
                println!("{}", self.get_reg::<i32>(10));
            }
            5 => {
                // read int
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                self.set_reg(10, buf.parse::<i32>().unwrap());
            }

            x => unimplemented!("Ecall {} is not implemented", x),
        }
    }
}
