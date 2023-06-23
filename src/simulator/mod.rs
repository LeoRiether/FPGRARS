//!
//! Runs a given RISC-V program instruction by instruction.
//!
//! Implemented instructions can be found at [Instructions](./parser/enum.Instruction.html),
//! and you can find how they're simulated at [Simulator::run](struct.Simulator.html#method.run)
//!

mod executor;
mod files;
mod into_register;
pub mod memory;
mod midi;
mod util;

use crate::parser;
use crate::renderer::{FRAME_0, FRAME_1, FRAME_SIZE};
use into_register::*;
use memory::*;
use owo_colors::OwoColorize;
use std::{mem, time};

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
    pub code: Vec<executor::Executor>,
    pub code_ctx: Vec<crate::parser::token::Context>,
}

impl Default for Simulator {
    fn default() -> Self {
        Self {
            registers: [0; 32],
            floats: [0.0; 32],
            status: Vec::new(),
            pc: 0,
            started_at: time::Instant::now(), // Will be set again in run()
            open_files: files::FileHolder::new(),
            midi_player: midi::MidiPlayer::default(),
            memory: Memory::new(),
            code: Vec::new(),
            code_ctx: Vec::new(),
        }
    }
}

impl Simulator {
    pub fn load_file(&mut self, path: &str) -> Result<(), parser::error::Error> {
        let parser::Parsed {
            code,
            code_ctx,
            data,
            globl,
        } = parser::parse(path, DATA_SIZE)?;

        self.code = executor::compile_all(&code);
        self.code_ctx = code_ctx;
        self.memory.data = data;

        if let Some(globl) = globl {
            self.pc = globl;
        }

        if crate::ARGS.print_instructions {
            eprintln!("{}", "Instructions: ---------------".bright_blue());
            code.iter().for_each(|i| eprintln!("{:?}", i));
            eprintln!("{}", "-----------------------------".bright_blue());
        }

        Ok(())
    }

    pub fn with_midi_port(mut self, midi_port: Option<usize>) -> Self {
        self.midi_player = midi::MidiPlayer::new(midi_port);
        self
    }

    pub fn with_memory(mut self, memory: Memory) -> Self {
        self.memory = memory;
        self
    }

    #[inline]
    fn reg<T: FromRegister>(&self, i: u8) -> T {
        FromRegister::from(unsafe { *self.registers.get_unchecked(i as usize) })
    }

    #[inline]
    fn set_reg<T: IntoRegister>(&mut self, i: u8, x: T) {
        if i != 0 {
            unsafe { *self.registers.get_unchecked_mut(i as usize) = x.into() };
        }
    }

    fn get_status(&self, i: u8) -> u32 {
        if i == parser::register_names::TIME_INDEX {
            self.started_at.elapsed().as_millis() as u32
        } else {
            self.status[i as usize]
        }
    }

    pub fn print_state(&self) {
        eprintln!("{}", "Registers:".bright_blue());
        for i in 0..32 {
            eprint!(
                "{}{:02}: {:08x} ",
                "x".bright_blue(),
                i.bright_blue(),
                self.registers[i]
            );
            if i % 4 == 3 {
                eprintln!();
            }
        }
        eprintln!();
        eprintln!("{}", "Float Registers:".bright_blue());
        for i in 0..32 {
            eprint!(
                "{}{:02}: {:<8} ",
                "f".bright_blue(),
                i.bright_blue(),
                self.floats[i]
            );
            if i % 4 == 3 {
                eprintln!();
            }
        }
        eprintln!();
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
        self.init();

        // Copy code to local variable so we can access it without borrowing self
        let code = mem::take(&mut self.code);

        executor::next(self, &code);

        if crate::ARGS.print_state {
            self.print_state();
        }
    }

    fn ecall(&mut self) -> EcallSignal {
        use crate::parser::register_names::*;
        use rand::{thread_rng, Rng};

        let a7 = self.reg::<u32>(17);

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
                print!("{}", self.reg::<i32>(10));
            }
            2 => {
                // print float
                print!("{}", self.floats[10]);
            }
            4 => {
                // print string
                let start = self.reg::<u32>(10) as usize; // a0
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
                let bytes = self.reg::<i32>(10); // a0

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
                print!("{}", self.reg::<u32>(10) as u8 as char);
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
                let t = self.reg::<u32>(10);
                std::thread::sleep(time::Duration::from_millis(t as u64));
            }

            34 => {
                // print hex int
                print!("{:#X}", self.reg::<u32>(10));
            }

            36 => {
                // print unsigned int
                print!("{}", self.reg::<u32>(10));
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
                let upper = self.reg::<u32>(11);
                self.set_reg(10, thread_rng().gen_range(0..upper));
            }
            43 => {
                // rand float in [0, 1)
                self.floats[10] = thread_rng().gen_range(0f32..1f32);
            }

            48 | 148 => {
                // clear screen
                let color = self.reg::<u8>(10); // a0
                let frame_select = self.reg::<u32>(11); // a1

                let mut mmio = self.memory.mmio.lock();
                let frame = if frame_select == 0 { FRAME_0 } else { FRAME_1 };
                for x in &mut mmio[frame..frame + FRAME_SIZE] {
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
