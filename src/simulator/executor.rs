//! Deals with how instructions are executed

use super::Simulator;
use crate::{instruction::Instruction, simulator::EcallSignal};

type ExecutorFn = dyn Fn(&mut Simulator, &[Executor]);

/// An Executor executes an instruction, moves the program counter forward (or appropriately, in
/// the case of a jump), and continues execution by calling the next executor
#[repr(transparent)]
pub struct Executor(Box<ExecutorFn>);

impl Executor {
    pub fn new<F: Fn(&mut Simulator, &[Executor]) + 'static>(f: F) -> Self {
        Self(Box::new(f))
    }

    pub fn call(&self, sim: &mut Simulator, next: &[Executor]) {
        (self.0)(sim, next);
    }
}

/// Execute the next instruction
#[inline(always)]
pub fn next(sim: &mut Simulator, code: &[Executor]) {
    let executor = code.get(sim.pc / 4).unwrap_or_else(|| {
        eprintln!(
            "Tried to access instruction at pc {:x}, but code is only {:x} bytes long",
            sim.pc,
            sim.code.len() * 4
        );
        std::process::exit(1);
    });

    executor.call(sim, code);
}

/// Compiles all instructions in a slice
pub fn compile_all(i: &[Instruction]) -> Vec<Executor> {
    i.iter().map(compile).collect()
}

/// Compiles a parsed instruction into an executor  
pub fn compile(i: &Instruction) -> Executor {
    use Instruction::*;

    match *i {
        Add(rd, rs1, rs2) => Executor::new(move |sim: &mut Simulator, code: &[Executor]| {
            sim.set_reg(rd, sim.get_reg::<i32>(rs1) + sim.get_reg::<i32>(rs2));
            sim.pc += 4;
            next(sim, code);
        }),

        // Type I
        Ecall => Executor::new(move |sim: &mut Simulator, code: &[Executor]| {
            use EcallSignal::*;
            match sim.ecall() {
                Exit => {} // don't execute the next instruction
                Continue => next(sim, code),
                Nothing => {
                    sim.pc += 4;
                    next(sim, code);
                }
            }
        }),

        Addi(rd, rs1, imm) => Executor::new(move |sim: &mut Simulator, code: &[Executor]| {
            sim.set_reg(rd, sim.get_reg::<i32>(rs1) + imm as i32);
            sim.pc += 4;
            next(sim, code);
        }),

        // Type SB + jumps
        Bne(rs1, rs2, label) => Executor::new(move |sim: &mut Simulator, code: &[Executor]| {
            if sim.get_reg::<i32>(rs1) != sim.get_reg::<i32>(rs2) {
                sim.pc = label;
            } else {
                sim.pc += 4;
            }
            next(sim, code);
        }),

        // Pseudoinstructions
        Li(rd, imm) => Executor::new(move |sim: &mut Simulator, code: &[Executor]| {
            sim.set_reg(rd, imm as i32);
            sim.pc += 4;
            next(sim, code);
        }),

        _ => todo!(),
    }
}
