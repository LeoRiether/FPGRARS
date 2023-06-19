//! Deals with how instructions are executed

use super::{into_register::IntoRegister, Simulator};
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
    let executor = code.get(sim.pc >> 2).unwrap_or_else(|| {
        eprintln!(
            "Tried to access instruction at pc {:x}, but code is only {:x} bytes long",
            sim.pc,
            sim.code.len() * 4
        );
        std::process::exit(1);
    });

    executor.call(sim, code);
}

/// Creates an executor that executes an instruction of type R.
/// PERF: if `op: F` where `F: Fn`, this is apparently inlined correctly, so performance is not
/// affected. However, if `op: fn(u32, u32) -> R`, performance is significantly worse! Like,
/// 50%-60% worse. This might be because generics are monomorphized, and each lambda is a different
/// Fn type.
/// TODO: Automatic benchmarks.
#[inline(always)]
fn exec_type_r<R, F>(rd: u8, rs1: u8, rs2: u8, op: F) -> Executor
where
    R: IntoRegister,
    F: Fn(u32, u32) -> R + 'static,
{
    Executor::new(move |sim, code| {
        sim.set_reg(rd, op(sim.reg(rs1), sim.reg(rs2)));
        sim.pc += 4;
        next(sim, code);
    })
}

/// Compiles all instructions in a slice
pub fn compile_all(i: &[Instruction]) -> Vec<Executor> {
    i.iter().map(compile).collect()
}

/// Compiles a parsed instruction into an executor  
pub fn compile(i: &Instruction) -> Executor {
    use Instruction::*;

    fn from_bool(x: bool) -> u32 {
        if x {
            1
        } else {
            0
        }
    }

    match *i {
        // Type R
        Add(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a.wrapping_add(b)),
        Sub(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a.wrapping_sub(b)),
        Sll(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a << (b & 0x1f)),
        Slt(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| from_bool((a as i32) < (b as i32))),
        Sltu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| from_bool(a < b)),
        Xor(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a ^ b),
        Srl(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a >> (b & 0x1f)),
        Sra(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| (a as i32) >> ((b as i32) & 0x1f)),
        Or(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a | b),
        And(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a & b),
        Mul(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a.wrapping_mul(b)),
        Div(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a.wrapping_div(b)),
        Divu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a / b),
        Rem(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a.wrapping_rem(b)),
        Remu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| a % b),

        // Type I
        Ecall => Executor::new(move |sim, code| {
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

        Addi(rd, rs1, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.reg::<i32>(rs1) + imm as i32);
            sim.pc += 4;
            next(sim, code);
        }),
        Slli(rd, rs1, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.reg::<i32>(rs1) << (imm & 0x1f));
            sim.pc += 4;
            next(sim, code);
        }),

        // Type S
        Sw(rs2, imm, rs1) => Executor::new(move |sim, code| {
            sim.memory.set_word(
                (sim.reg::<u32>(rs1).wrapping_add(imm)) as usize,
                sim.reg::<u32>(rs2),
            );
            sim.pc += 4;
            next(sim, code);
        }),

        // Type SB + jumps
        Bne(rs1, rs2, label) => Executor::new(move |sim, code| {
            if sim.reg::<i32>(rs1) != sim.reg::<i32>(rs2) {
                sim.pc = label;
            } else {
                sim.pc += 4;
            }
            next(sim, code);
        }),
        Blt(rs1, rs2, label) => Executor::new(move |sim, code| {
            if sim.reg::<i32>(rs1) < sim.reg::<i32>(rs2) {
                sim.pc = label;
            } else {
                sim.pc += 4;
            }
            next(sim, code);
        }),
        Bge(rs1, rs2, label) => Executor::new(move |sim, code| {
            if sim.reg::<i32>(rs1) >= sim.reg::<i32>(rs2) {
                sim.pc = label;
            } else {
                sim.pc += 4;
            }
            next(sim, code);
        }),

        Jalr(rd, rs1, imm) => Executor::new(move |sim, code| {
            // This produces a weird result for `jalr s0 s0 0`. s0 is set to pc+4 before the jump occurs
            // so it works as a nop. Maybe this is correct, maybe it's not, but I'll copy the behavior seen in
            // RARS to be consistent.
            sim.set_reg(rd, (sim.pc + 4) as u32);
            sim.pc = (sim.reg::<i32>(rs1) + (imm as i32)) as usize & !1;
            next(sim, code);
        }),
        Jal(rd, label) => Executor::new(move |sim, code| {
            sim.set_reg(rd, (sim.pc + 4) as u32);
            sim.pc = label;
            next(sim, code);
        }),

        // Type I -- Loads
        Lw(rd, imm, rs1) => Executor::new(move |sim, code| {
            let data = sim
                .memory
                .get_word((sim.reg::<i32>(rs1) + imm as i32) as usize);
            sim.set_reg(rd, data);
            sim.pc += 4;
            next(sim, code);
        }),

        // Pseudoinstructions
        Li(rd, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, imm as i32);
            sim.pc += 4;
            next(sim, code);
        }),

        Mv(rd, rs1) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.reg::<u32>(rs1));
            sim.pc += 4;
            next(sim, code);
        }),

        _ => todo!("Instruction {:?}", *i),
    }
}
