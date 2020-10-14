use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::{Arc, Mutex};

const DATA_SIZE: usize = 128;
const MMIO_SIZE: usize = 2 * 320 * 2 * 240 * 2 + 4;
const MMIO_START: usize = 0xff000000;

pub mod parser;

mod into_register;
use into_register::*;

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

    pub fn set_byte(&mut self, i: usize, x: u8) {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            (*mmio)[i - MMIO_START] = x;
        } else {
            self.data[i] = x;
        }
    }
}

pub struct Simulator {
    registers: [u32; 32],
    floats: [f32; 32],
    status: Vec<u32>, // I'm not sure myself how many status register I'll use
    pc: usize,

    pub memory: Memory,
    pub code: Vec<parser::Instruction>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            floats: [0.0; 32],
            status: Vec::new(),
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

    pub fn load_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, parser::Error> {
        let parser::Parsed { code, data } = parser::parse_file(path)?;
        self.code = code;
        self.memory.data = data;
        self.memory.data.resize(DATA_SIZE, 0);
        Ok(self)
    }

    pub fn run(&mut self) {
        use parser::Instruction::*;
        loop {
            match self.code[self.pc / 4] {
                // Type R
                Add(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) + self.get_reg::<i32>(rs2))
                }

                // Type I
                Ecall => self.ecall(),

                Addi(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) + imm),

                // Type S
                Sb(rs2, imm, rs1) => self.memory.set_byte(
                    (self.get_reg::<i32>(rs1) + imm) as u32 as usize,
                    self.get_reg::<u8>(rs2),
                ),

                // Type SB + jumps
                Bge(rs1, rs2, label) => {
                    if self.get_reg::<i32>(rs1) >= self.get_reg::<i32>(rs2) {
                        self.pc = label;
                        continue;
                    }
                }

                Jal(rd, label) => {
                    self.set_reg(rd, (self.pc + 4) as u32);
                    self.pc = label;
                    continue;
                }

                // Pseudoinstructions
                Li(rd, imm) => self.set_reg(rd, imm),
                Mv(rd, rs1) => self.registers[rd as usize] = self.registers[rs1 as usize],

                _ => unimplemented!(),
            }

            self.pc += 4;
        }
    }

    fn ecall(&mut self) {
        match self.get_reg::<i32>(17) {
            // 17 = a7
            10 => std::process::exit(0),

            x => unimplemented!("Ecall {} is not implemented", x),
        }
    }
}
