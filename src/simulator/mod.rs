//!
//! Runs a given RISC-V program instruction by instruction.
//!
//! Implemented instructions can be found at [Instructions](./parser/enum.Instruction.html),
//! and you can find how they're simulated at [Simulator::run](struct.Simulator.html#method.run)
//!

use std::time;

use crate::codegen::{self, constants::*};
use crate::parser::{self, Includable, MacroParseable, RISCVParser};
use crate::renderer::{FRAME_0, FRAME_1, HEIGHT, WIDTH};

mod memory;
use memory::*;

mod into_register;
use into_register::*;

mod files;
mod midi;

mod util;

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
    status: Vec<u32>, // I'm not sure myself how many status registers I'll use
    pc: usize,
    started_at: time::Instant,

    open_files: files::FileHolder,
    midi_player: midi::MidiPlayer,

    pub memory: Memory,
    pub code: Vec<parser::Instruction>,
}

impl Simulator {
    pub fn new(midi_port: Option<usize>) -> Self {
        Self {
            registers: [0; 32],
            floats: [0.0; 32],
            status: Vec::new(),
            pc: 0,
            started_at: time::Instant::now(), // Will be set again in run()
            open_files: files::FileHolder::new(),
            midi_player: midi::MidiPlayer::new(midi_port),
            memory: Memory::new(),
            code: Vec::new(),
        }
    }

    fn get_reg<T: FromRegister>(&self, i: u8) -> T {
        let value = if cfg!(debug_assertions) {
            self.registers[i as usize]
        } else {
            unsafe { *self.registers.get_unchecked(i as usize) }
        };

        FromRegister::from(value)
    }

    fn set_reg<T: IntoRegister>(&mut self, i: u8, x: T) {
        // This could be made branchless by setting reg[i] = i == 0 ? 0 : x, but I'm not sure it's worth it
        if i != 0 {
            if cfg!(debug_assertions) {
                self.registers[i as usize] = x.into();
            } else {
                unsafe {
                    *self.registers.get_unchecked_mut(i as usize) = x.into();
                }
            };
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

        let from_bool = |b| if b { 1 } else { 0 };

        macro_rules! get {
            ($reg:ident $type:ty) => {
                self.get_reg::<$type>($reg)
            };
        }

        macro_rules! set {
            ($rd:ident = $val:expr) => {
                self.set_reg($rd, $val)
            };
        }

        self.init();

        ////////////////////////////////////////////////////////////////////////////////
        let old_code = std::mem::take(&mut self.code);
        let machine_code: Vec<codegen::Instruction> = old_code
            .iter()
            .zip((0..).step_by(4))
            .map(|(instruction, pc)| codegen::Instruction::from_parsed(instruction.clone(), pc * 4))
            .collect();

        println!("Decoded instructions:");
        for x in &machine_code {
            if x.0 != 0 {
                println!("{:x}", x.0);
            }
        }

        ////////////////////////////////////////////////////////////////////////////////

        loop {
            let instr = machine_code.get(self.pc / 4).unwrap_or_else(|| {
                panic!(
                    "Tried to execute an instruction outside of the code segment: pc = {}",
                    self.pc,
                )
            });

            let funct3 = instr.funct3();
            let funct7 = instr.funct7();
            let funct10 = instr.funct10();

            macro_rules! covered {
                () => {
                    panic!(
                        "{:?} should have been covered by the new format already",
                        instr
                    )
                };
            }

            match instr.opcode() {
                OPCODE_TYPE_R => {
                    let (rd, rs1, rs2) = (instr.rd() as u8, instr.rs1() as u8, instr.rs2() as u8);
                    match funct10 {
                        add::F10 => set! { rd = get!(rs1 i32) + get!(rs2 i32) },
                        sub::F10 => set! { rd = get!(rs1 i32) - get!(rs2 i32) },
                        sll::F10 => set! { rd = get!(rs1 u32) << (get!(rs2 i32) & 0x1f) },
                        slt::F10 => set! { rd = from_bool(get!(rs1 i32) < get!(rs2 i32)) },
                        sltu::F10 => set! { rd = from_bool(get!(rs1 u32) < get!(rs2 u32)) },
                        xor::F10 => set! { rd = get!(rs1 u32) ^ get!(rs2 u32) },
                        srl::F10 => set! { rd = get!(rs1 u32) >> (get!(rs2 u32) & 0x1f) },
                        sra::F10 => set! { rd = get!(rs1 i32) >> (get!(rs2 u32) & 0x1f) },
                        or::F10 => set! { rd = get!(rs1 u32) | get!(rs2 u32) },
                        and::F10 => set! { rd = get!(rs1 u32) & get!(rs2 u32) },
                        mul::F10 => set! { rd = get!(rs1 i32) * get!(rs2 i32) },
                        // mulh::F10 => set! { rd = (get!(rs1 i64) * get!(rs2 i64)) as i32 },
                        // mulhsu::F10 => set! { rd = (get!(rs1 i64) * get!(rs2 u64)) as i32 },
                        // mulhu::F10 => set! { rd = (get!(rs1 u64) * get!(rs2 u64)) as i32 },
                        div::F10 => set! { rd = get!(rs1 i32) / get!(rs2 i32) },
                        divu::F10 => set! { rd = get!(rs1 u32) / get!(rs2 u32) },
                        rem::F10 => set! { rd = get!(rs1 i32) % get!(rs2 i32) },
                        remu::F10 => set! { rd = get!(rs1 u32) % get!(rs2 u32) },
                        _ => panic!("Unknown TypeR instruction: {:x}", instr.0),
                    }
                }

                OPCODE_TYPE_I_IMM => {
                    let (rd, rs1) = (instr.rd() as u8, instr.rs1() as u8);
                    let imm = instr.imm_i();
                    match funct3 {
                        addi::F3 => set! { rd = get!(rs1 i32) + imm },
                        slti::F3 => set! { rd = from_bool(get!(rs1 i32) < imm) },
                        // TODO: implement this correctly
                        // sltiu::F3 => set! { rd = from_bool(get!(rs1 u32) < imm) },
                        xori::F3 => set! { rd = get!(rs1 u32) ^ imm as u32 },
                        ori::F3 => set! { rd = get!(rs1 u32) | imm as u32 },
                        andi::F3 => set! { rd = get!(rs1 u32) & imm as u32 },
                        // TODO: figure out why slli has a funct7
                        slli::F3 => set! { rd = get!(rs1 u32) << (imm & 0x1f) },
                        srli::F3 => match funct7 {
                            srli::F7 => set! { rd = get!(rs1 u32) >> (imm & 0x1f) },
                            srai::F7 => set! { rd = get!(rs1 i32) >> (imm & 0x1f) },
                            _ => panic!("Unknown TypeI instruction: {:x}", instr.0),
                        },
                        _ => panic!("Unknown TypeI instruction: {:x}", instr.0),
                    }
                }

                OPCODE_TYPE_I_SYSTEM => {
                    let rs2 = instr.rs2();
                    match funct10 {
                        ecall::F10 if rs2 == 0 => {
                            use EcallSignal::*;
                            match self.ecall() {
                                Exit => return,
                                Continue => continue,
                                Nothing => {}
                            }
                        }
                        _ => panic!("Unknown TypeI::System instruction: {:x}", instr.0),
                    }
                }

                OPCODE_TYPE_I_LOAD => {
                    let (rd, rs1) = (instr.rd() as u8, instr.rs1() as u8);
                    let imm = instr.imm_i();
                    match funct3 {
                        lb::F3 => {
                            set! { rd = self.memory.get_byte(get!(rs1 u32).wrapping_add(imm as u32) as usize) as i32 }
                        }
                        lh::F3 => {
                            set! { rd = self.memory.get_half(get!(rs1 u32).wrapping_add(imm as u32) as usize) as i32 }
                        }
                        lw::F3 => {
                            set! { rd = self.memory.get_word(get!(rs1 u32).wrapping_add(imm as u32) as usize) }
                        }
                        lbu::F3 => {
                            set! { rd = self.memory.get_byte(get!(rs1 u32).wrapping_add(imm as u32) as usize) as u32 }
                        }
                        lhu::F3 => {
                            set! { rd = self.memory.get_half(get!(rs1 u32).wrapping_add(imm as u32) as usize) as u32 }
                        }
                        _ => panic!("Unknown TypeI::Load instruction: {:x}", instr.0),
                    }
                }

                OPCODE_TYPE_I_JALR => {
                    let (rd, rs1, imm) = (instr.rd() as u8, instr.rs1() as u8, instr.imm_i());
                    // This produces a weird result for `jalr s0 s0 0`. s0 is set to pc+4 before the jump occurs
                    // so it works as a nop. Maybe this is correct, maybe it's not, but I'll copy the behavior seen in
                    // RARS to be consistent.
                    set! { rd = (self.pc + 4) as u32 };
                    self.pc = (get!(rs1 i32) + imm) as usize & !1;
                    continue;
                }

                OPCODE_TYPE_S => {
                    let (rs1, rs2) = (instr.rs1() as u8, instr.rs2() as u8);
                    let imm = instr.imm_s();
                    match funct3 {
                        sb::F3 => {
                            self.memory.set_byte(
                                get!(rs1 u32).wrapping_add(imm as u32) as usize,
                                get!(rs2 u8),
                            );
                        }
                        sh::F3 => {
                            self.memory.set_half(
                                get!(rs1 u32).wrapping_add(imm as u32) as usize,
                                get!(rs2 u16),
                            );
                        }
                        sw::F3 => {
                            self.memory.set_word(
                                get!(rs1 u32).wrapping_add(imm as u32) as usize,
                                get!(rs2 u32),
                            );
                        }
                        _ => panic!(
                            "Unknown TypeS instruction: {:?} (machine code {:x})",
                            old_code[self.pc / 4],
                            instr.0
                        ),
                    }
                }

                OPCODE_TYPE_B => {
                    let (rs1, rs2) = (instr.rs1() as u8, instr.rs2() as u8);
                    let imm = instr.imm_b() as u32;
                    match funct3 {
                        beq::F3 => {
                            if get!(rs1 u32) == get!(rs2 u32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        bne::F3 => {
                            if get!(rs1 u32) != get!(rs2 u32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        blt::F3 => {
                            if get!(rs1 i32) < get!(rs2 i32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        bge::F3 => {
                            if get!(rs1 i32) >= get!(rs2 i32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        bltu::F3 => {
                            if get!(rs1 u32) < get!(rs2 u32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        bgeu::F3 => {
                            if get!(rs1 u32) >= get!(rs2 u32) {
                                self.pc = (self.pc as u32).wrapping_add(imm) as usize;
                                continue;
                            }
                        }
                        _ => panic!("Unknown TypeB instruction: {:x}", instr.0),
                    }
                }

                _ => match old_code[self.pc / 4] {
                    // Type R
                    Add(..) | Sub(..) | Sll(..) | Slt(..) | Sltu(..) | Xor(..) | Srl(..)
                    | Sra(..) | Or(..) | And(..) | Mul(..) | Div(..) | Divu(..) | Rem(..)
                    | Remu(..) => covered!(),

                    // Type I -- Immediate
                    Addi(..) | Slli(..) | Slti(..) | Sltiu(..) | Xori(..) | Srli(..) | Srai(..)
                    | Ori(..) | Andi(..) => covered!(),

                    // Type I -- System
                    Ecall => covered!(),

                    // Type I, loads from memory
                    Lb(..) | Lh(..) | Lw(..) | Lbu(..) | Lhu(..) => covered!(),

                    Float(F::Lw(rd, imm, rs1)) => {
                        let rd = rd as usize;
                        let x = self
                            .memory
                            .get_float(self.get_reg::<u32>(rs1).wrapping_add(imm) as usize);
                        self.floats[rd] = x;
                    }

                    // Type S
                    Sb(..) | Sh(..) | Sw(..) => covered!(),

                    Float(F::Sw(rs2, imm, rs1)) => {
                        let x = self.floats[rs2 as usize];
                        self.memory
                            .set_float(self.get_reg::<u32>(rs1).wrapping_add(imm) as usize, x);
                    }

                    // Type B
                    Beq(..) | Bne(..) | Blt(..) | Bge(..) | Bltu(..) | Bgeu(..) => covered!(),

                    // Type I -- JALR
                    Jalr(..) => covered!(),

                    Jal(rd, label) => {
                        set! { rd = (self.pc + 4) as u32 };
                        self.pc = label;
                        continue;
                    }

                    // CSR
                    CsrRw(rd, fcsr, rs1) => {
                        set! { rd = self.get_status(fcsr) };
                        self.status[fcsr as usize] = self.get_reg::<u32>(rs1);
                    }
                    CsrRwi(rd, fcsr, imm) => {
                        set! { rd = self.get_status(fcsr) };
                        self.status[fcsr as usize] = imm;
                    }
                    CsrRs(rd, fcsr, rs1) => {
                        set! { rd = self.get_status(fcsr) };
                        self.status[fcsr as usize] |= self.get_reg::<u32>(rs1);
                    }
                    CsrRsi(rd, fcsr, imm) => {
                        set! { rd = self.get_status(fcsr) };
                        self.status[fcsr as usize] |= imm;
                    }
                    CsrRc(rd, fcsr, rs1) => {
                        set! { rd = self.get_status(fcsr) };
                        self.status[fcsr as usize] &= !self.get_reg::<u32>(rs1);
                    }
                    CsrRci(rd, fcsr, imm) => {
                        set! { rd = self.get_status(fcsr) };
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
                        self.set_reg(rd, from_bool(self.floats[rs1] == self.floats[rs2]));
                    }
                    Float(F::Le(rd, rs1, rs2)) => {
                        let (rs1, rs2) = (rs1 as usize, rs2 as usize);
                        self.set_reg(rd, from_bool(self.floats[rs1] <= self.floats[rs2]));
                    }
                    Float(F::Lt(rd, rs1, rs2)) => {
                        let (rs1, rs2) = (rs1 as usize, rs2 as usize);
                        self.set_reg(rd, from_bool(self.floats[rs1] < self.floats[rs2]));
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
                    URet => {
                        use crate::parser::register_names::UEPC_INDEX;
                        self.pc = self.status[UEPC_INDEX as usize] as usize;
                        continue;
                    }
                },
            };

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

        if self.midi_player.handle_ecall(a7, &mut self.registers) {
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
            2 => {
                // print float
                print!("{}", self.floats[10]);
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
                // read float
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).unwrap();
                self.floats[10] = buf.trim().parse::<f32>().unwrap();
            }

            9 => {
                // sbrk
                // Like RARS, negative increments are not allowed
                let bytes = self.get_reg::<i32>(10); // a0

                if bytes < 0 {
                    panic!("`sbrk` does not allow negative increments");
                }

                let padding = (4 - bytes % 4) % 4; // makes sure we're always allocating full words
                let bytes = (bytes + padding) as usize;

                self.set_reg(10, (HEAP_START + self.memory.dynamic.len()) as u32);

                self.memory.dynamic.reserve(bytes); // may reserve more than `bytes`
                self.memory
                    .dynamic
                    .resize(self.memory.dynamic.len() + bytes, 0);
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
