//!
//! Runs a given RISC-V program instruction by instruction.
//!
//! Implemented instructions can be found at [Instructions](./parser/enum.Instruction.html),
//! and you can find how they're simulated at [Simulator::run](struct.Simulator.html#method.run)
//!

use std::sync::{Arc, Mutex};
use std::time;

const DATA_SIZE: usize = 0x0040_0000; // TODO: this, but I think it's about this much
const MMIO_SIZE: usize = 0x0021_0000;
const MMIO_START: usize = 0xff00_0000;
const KBMMIO_CONTROL: usize = 0xff20_0000;
const KBMMIO_DATA: usize = 0xff20_0004;

use crate::renderer::{FRAME_0, FRAME_1, HEIGHT, WIDTH};
const VIDEO_START: usize = MMIO_START + FRAME_0;
const VIDEO_END: usize = MMIO_START + FRAME_1 + WIDTH * HEIGHT;

use crate::parser::{self, Includable, MacroParseable, RISCVParser};

mod into_register;
use into_register::*;

mod files;

mod util;

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

    /// Sets N bytes in the video memory, but ignores bytes equal to 0xC7.
    fn set_with_transparency(&mut self, i: usize, mut x: u32, n: usize) -> bool {
        if i < VIDEO_START || i >= VIDEO_END {
            return false;
        }

        let mut mmio = self.mmio.lock().unwrap();
        let i = i - MMIO_START;

        for data in &mut mmio[i..i+n] {
            let byte = x as u8;
            if byte != 0xC7 {
                *data = byte;
            }

            x >>= 8;
        }

        true
    }

    pub fn get_with<T, F>(&self, i: usize, read: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            if i == KBMMIO_DATA {
                mmio[KBMMIO_CONTROL - MMIO_START] = 0;
            }
            read(&mmio[i - MMIO_START..])
        } else {
            read(&self.data[i..])
        }
    }

    pub fn set_with<T, F, R>(&mut self, i: usize, x: T, write: F) -> R
    where
        F: FnOnce(&mut [u8], T) -> R,
    {
        if i >= MMIO_START {
            let mut mmio = self.mmio.lock().unwrap();
            write(&mut mmio[i - MMIO_START..], x)
        } else {
            write(&mut self.data[i..], x)
        }
    }

    pub fn get_byte(&self, i: usize) -> u8 {
        self.get_with(i, |v| v[0])
    }

    pub fn set_byte(&mut self, i: usize, x: u8) {
        if self.set_with_transparency(i, x as u32, 1) {
            return;
        }
        self.set_with(i, x, |v, x| v[0] = x)
    }

    pub fn get_half(&self, i: usize) -> u16 {
        self.get_with(i, LittleEndian::read_u16)
    }

    pub fn set_half(&mut self, i: usize, x: u16) {
        if self.set_with_transparency(i, x as u32, 2) {
            return;
        }
        self.set_with(i, x, LittleEndian::write_u16)
    }

    pub fn get_word(&self, i: usize) -> u32 {
        self.get_with(i, LittleEndian::read_u32)
    }

    pub fn set_word(&mut self, i: usize, x: u32) {
        if self.set_with_transparency(i, x, 4) {
            return;
        }
        self.set_with(i, x, LittleEndian::write_u32)
    }

    pub fn get_float(&self, i: usize) -> f32 {
        self.get_with(i, LittleEndian::read_f32)
    }

    pub fn set_float(&mut self, i: usize, x: f32) {
        self.set_with(i, x, LittleEndian::write_f32)
    }
}

/// Returned by the [ecall](struct.Simulator.html#method.ecall) procedure
enum EcallSignal {
    Nothing,
    Exit,
    Continue,
}

/// Simulates a RISC-V CPU. Generally initialized by calling [load_from_file](struct.Simulator.html#method.load_from_file)
/// and ran by calling [run](struct.Simulator.html#method.run).
pub struct Simulator {
    registers: [u32; 32],
    floats: [f32; 32],
    status: Vec<u32>, // I'm not sure myself how many status register I'll use
    pc: usize,
    started_at: time::Instant,

    open_files: files::FileHolder,

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
            started_at: time::Instant::now(), // Will be set again in run()
            open_files: files::FileHolder::new(),
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

    fn get_status(&self, i: u8) -> u32 {
        if i == parser::register_names::TIME_INDEX {
            self.started_at.elapsed().as_millis() as u32
        } else {
            self.status[i as usize]
        }
    }

    pub fn load_from_file(mut self, path: String) -> Result<Self, parser::Error> {
        // TODO: some of this logic is duplicated from the Includer, try to dedup?
        let pathbuf = std::path::PathBuf::from(&path);
        let error = format!("Can't open file: <{:?}>", pathbuf.to_str());
        let parser::Parsed { code, data } = parser::file_lines(&path)
            .expect(&error)
            .parse_includes(pathbuf)
            .parse_macros()
            .parse_riscv(DATA_SIZE)?;

        self.code = code;
        self.memory.data = data;

        Ok(self)
    }

    fn init(&mut self) {
        // Create necessary status registers
        self.status
            .resize(parser::register_names::status().len(), 0);

        // Set stack pointer
        self.set_reg(2, self.memory.data.len() as u32 - 4);

        // Set global pointer
        self.set_reg(3, 0x10008000);

        self.started_at = time::Instant::now();
        self.status[parser::register_names::MISA_INDEX as usize] = 0x40001128;
    }

    pub fn run(&mut self) {
        use parser::FloatInstruction as F;
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

        self.init();

        loop {
            match self.code[self.pc / 4] {
                // Type R
                Add(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) + self.get_reg::<i32>(rs2))
                }
                Sub(rd, rs1, rs2) => {
                    self.set_reg(rd, self.get_reg::<i32>(rs1) - self.get_reg::<i32>(rs2))
                }
                Sll(rd, rs1, rs2) => self.set_reg(
                    rd,
                    self.get_reg::<u32>(rs1) << (self.get_reg::<i32>(rs2) & 0x1f),
                ),
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
                Srl(rd, rs1, rs2) => self.set_reg(
                    rd,
                    self.get_reg::<u32>(rs1) >> (self.get_reg::<i32>(rs2) & 0x1f),
                ),
                Sra(rd, rs1, rs2) => self.set_reg(
                    rd,
                    self.get_reg::<i32>(rs1) >> (self.get_reg::<i32>(rs2) & 0x1f),
                ),
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
                Ecall => {
                    use EcallSignal::*;
                    match self.ecall() {
                        Exit => {
                            return;
                        }
                        Continue => {
                            continue;
                        }
                        Nothing => {}
                    }
                }
                Addi(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) + (imm as i32)),
                Slli(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) << (imm & 0x1f)),
                Slti(rd, rs1, imm) => {
                    self.set_reg(rd, to_1(self.get_reg::<i32>(rs1) < (imm as i32)))
                }
                Sltiu(rd, rs1, imm) => self.set_reg(rd, to_1(self.get_reg::<u32>(rs1) < imm)),
                Xori(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) ^ imm),
                Srli(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) >> (imm & 0x1f)),
                Srai(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<i32>(rs1) >> (imm & 0x1f)),
                Ori(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) | imm),
                Andi(rd, rs1, imm) => self.set_reg(rd, self.get_reg::<u32>(rs1) & imm),

                // Type I, loads from memory
                Lb(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_byte((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as i8 as u32,
                ),
                Lh(rd, imm, rs1) => self.set_reg(
                    rd,
                    self.memory
                        .get_half((self.get_reg::<u32>(rs1).wrapping_add(imm)) as usize)
                        as i16 as u32,
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
                Float(F::Lw(rd, imm, rs1)) => {
                    let rd = rd as usize;
                    let x = self
                        .memory
                        .get_float(self.get_reg::<u32>(rs1).wrapping_add(imm) as usize);
                    self.floats[rd] = x;
                }

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
                Float(F::Sw(rs2, imm, rs1)) => {
                    let x = self.floats[rs2 as usize];
                    self.memory
                        .set_float(self.get_reg::<u32>(rs1).wrapping_add(imm) as usize, x);
                }

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
                    // This produces a weird result for `jalr s0 s0 0`. s0 is set to pc+4 before the jump occurs
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

                // CSR
                CsrRw(rd, fcsr, rs1) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] = self.get_reg::<u32>(rs1);
                }
                CsrRwi(rd, fcsr, imm) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] = imm;
                }
                CsrRs(rd, fcsr, rs1) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] |= self.get_reg::<u32>(rs1);
                }
                CsrRsi(rd, fcsr, imm) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] |= imm;
                }
                CsrRc(rd, fcsr, rs1) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] &= !self.get_reg::<u32>(rs1);
                }
                CsrRci(rd, fcsr, imm) => {
                    self.set_reg(rd, self.get_status(fcsr));
                    self.status[fcsr as usize] &= !imm;
                }

                // Floating point
                Float(F::Add(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1] + self.floats[rs2];
                }
                Float(F::Sub(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1] - self.floats[rs2];
                }
                Float(F::Mul(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1] * self.floats[rs2];
                }
                Float(F::Div(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1] / self.floats[rs2];
                }
                Float(F::Equ(rd, rs1, rs2)) => {
                    let (rs1, rs2) = (rs1 as usize, rs2 as usize);
                    self.set_reg(rd, to_1(self.floats[rs1] == self.floats[rs2]));
                }
                Float(F::Le(rd, rs1, rs2)) => {
                    let (rs1, rs2) = (rs1 as usize, rs2 as usize);
                    self.set_reg(rd, to_1(self.floats[rs1] <= self.floats[rs2]));
                }
                Float(F::Lt(rd, rs1, rs2)) => {
                    let (rs1, rs2) = (rs1 as usize, rs2 as usize);
                    self.set_reg(rd, to_1(self.floats[rs1] < self.floats[rs2]));
                }
                Float(F::Max(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1].max(self.floats[rs2]);
                }
                Float(F::Min(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1].min(self.floats[rs2]);
                }
                Float(F::SgnjS(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1].copysign(self.floats[rs2]);
                }
                Float(F::SgnjNS(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    self.floats[rd] = self.floats[rs1].copysign(-self.floats[rs2]);
                }
                Float(F::SgnjXS(rd, rs1, rs2)) => {
                    let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
                    let (a, b) = (self.floats[rs1], self.floats[rs2]);

                    // I'm pretty sure this is correct (for most architectures anyway)
                    self.floats[rd] = f32::from_bits(a.to_bits() ^ (b.to_bits() & (1 << 31)));
                }

                // I didn't even know this existed before this project
                Float(F::Class(rd, rs1)) => {
                    let rs1 = rs1 as usize;
                    self.set_reg(rd, util::class_mask(self.floats[rs1]));
                }

                Float(F::CvtSW(rd, rs1)) => {
                    let rd = rd as usize;
                    self.floats[rd] = self.get_reg::<i32>(rs1) as f32;
                }
                Float(F::CvtSWu(rd, rs1)) => {
                    let rd = rd as usize;
                    self.floats[rd] = self.get_reg::<u32>(rs1) as f32;
                }
                Float(F::CvtWS(rd, rs1)) => {
                    let rs1 = rs1 as usize;
                    self.set_reg(rd, self.floats[rs1] as i32);
                }
                Float(F::CvtWuS(rd, rs1)) => {
                    let rs1 = rs1 as usize;
                    self.set_reg(rd, self.floats[rs1] as u32);
                }

                Float(F::MvSX(rd, rs1)) => {
                    let rd = rd as usize;
                    self.floats[rd] = f32::from_bits(self.get_reg::<u32>(rs1));
                }
                Float(F::MvXS(rd, rs1)) => {
                    let rs1 = rs1 as usize;
                    self.set_reg(rd, self.floats[rs1].to_bits());
                }

                Float(F::Sqrt(rd, rs1)) => {
                    let (rd, rs1) = (rd as usize, rs1 as usize);
                    self.floats[rd] = self.floats[rs1].sqrt();
                }

                // Pseudoinstructions
                Li(rd, imm) => self.set_reg(rd, imm),
                Mv(rd, rs1) => self.registers[rd as usize] = self.registers[rs1 as usize],
                Ret => {
                    self.pc = self.registers[1] as usize;
                    continue;
                }
                URet => {
                    use crate::parser::register_names::UEPC_INDEX;
                    self.pc = self.status[UEPC_INDEX as usize] as usize;
                    continue;
                }
            }

            self.pc += 4;
        }
    }

    fn ecall(&mut self) -> EcallSignal {
        use crate::parser::register_names::*;
        use rand::{thread_rng, Rng};

        let a7 = self.get_reg::<u32>(17);

        if files::handle_ecall(
            a7,
            &mut self.open_files,
            &mut self.registers,
            &mut self.memory,
        ) {
            return EcallSignal::Nothing;
        }

        match a7 {
            10 => return EcallSignal::Exit,
            110 => loop {
                std::thread::sleep(time::Duration::from_millis(500));
            },
            1 => {
                // print int
                print!("{}", self.get_reg::<i32>(10));
            }
            4 => {
                // print string
                let start = self.get_reg::<u32>(10) as usize; // a0
                (start..)
                    .map(|i| self.memory.get_byte(i) as char)
                    .take_while(|&c| c != '\0')
                    .for_each(|c| print!("{}", c));
            }
            5 => {
                // read int
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                self.set_reg(10, buf.trim().parse::<i32>().unwrap());
            }
            6 => {
                // print float
                print!("{}", self.floats[10]);
            }
            11 => {
                // print char
                print!("{}", self.get_reg::<u32>(10) as u8 as char);
            }

            30 => {
                // get time
                let epoch = time::SystemTime::UNIX_EPOCH;
                let duration = time::SystemTime::now().duration_since(epoch).unwrap();
                let ms = duration.as_millis() as u64;
                self.set_reg(10, ms as u32);
                self.set_reg(11, (ms >> 32) as u32);
            }

            31 | 33 => {
                // midi stuff, but nops for now
            }

            32 => {
                // sleep ms
                let t = self.get_reg::<u32>(10);
                std::thread::sleep(time::Duration::from_millis(t as u64));
            }

            34 => {
                // print hex int
                print!("{:#X}", self.get_reg::<u32>(10));
            }

            36 => {
                // print unsigned int
                print!("{}", self.get_reg::<u32>(10));
            }

            // RNG stuff
            40 => {
                // TODO: seed the RNG
            }
            41 => {
                // rand int
                self.set_reg(10, thread_rng().gen::<i32>());
            }
            42 => {
                // rand int in [0, a1)
                let upper = self.get_reg::<u32>(11);
                self.set_reg(10, thread_rng().gen_range::<u32, _, _>(0, upper));
            }
            43 => {
                // rand float in [0, 1)
                self.floats[10] = thread_rng().gen_range(0f32, 1f32);
            }

            48 | 148 => {
                // clear screen
                let color = self.get_reg::<u8>(10); // a0
                let frame_select = self.get_reg::<u32>(11); // a1

                let mut mmio = self.memory.mmio.lock().unwrap();
                let frame = if frame_select == 0 { FRAME_0 } else { FRAME_1 };
                for x in &mut mmio[frame..frame + WIDTH * HEIGHT] {
                    *x = color;
                }
            }

            // These two should only be here temporarily for convenience
            0xff00 => {
                self.floats[10] = self.floats[10].sin();
            }
            0xff01 => {
                self.floats[10] = self.floats[10].cos();
            }

            // Does the user want to handle this ecall?
            _x if self.status[USTATUS_INDEX as usize] & 1 == 1 => {
                self.status[UCAUSE_INDEX as usize] = 8; // ecall exception
                self.status[UEPC_INDEX as usize] = self.pc as u32; // set uret location
                self.pc = self.status[UTVEC_INDEX as usize] as usize; // jump to utvec
                return EcallSignal::Continue;
            }

            x => unimplemented!("Ecall {} is not implemented", x),
        }

        EcallSignal::Nothing
    }
}
