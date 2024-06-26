//! Deals with how instructions are executed

use owo_colors::OwoColorize;

use super::{into_register::IntoRegister, Simulator};
use crate::{
    instruction::{FloatInstruction, Instruction},
    simulator::{util::class_mask, EcallSignal},
};

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

fn display_memory_out_of_bounds_error(sim: &Simulator, position: usize) -> ! {
    let byte_msg = if position as i32 >= 0 {
        format!("{}", position.bright_blue())
    } else {
        format!(
            "{} (the same as {} if signed)",
            position.bright_blue(),
            (position as i32).bright_blue()
        )
    };
    eprintln!(
        "{} Out of bounds memory access at byte {}!\n{}: when executing instruction\n{}",
        "   [error]".bright_red(),
        byte_msg,
        "   Note".bright_yellow(),
        sim.code_ctx[sim.pc >> 2]
    );
    std::process::exit(1);
}

fn display_instruction_out_of_bounds_error(sim: &Simulator, new_pc: usize) -> ! {
    eprintln!(
            "Tried to access instruction at pc {:x}, but code is only {:x} bytes long.\nLast instruction executed: {}",
            new_pc,
            sim.code.len() * 4,
            sim.code_ctx[sim.pc >> 2],
        );
    std::process::exit(1);
}

/// Execute the next instruction
#[inline(always)]
pub fn next(sim: &mut Simulator, code: &[Executor], new_pc: usize) {
    if let Some(position) = sim.memory.out_of_bounds_access {
        display_memory_out_of_bounds_error(sim, position);
    }

    let executor = code
        .get(new_pc >> 2)
        .unwrap_or_else(|| display_instruction_out_of_bounds_error(sim, new_pc));

    sim.pc = new_pc;
    executor.call(sim, code);
}

fn from_bool(x: bool) -> u32 {
    if x {
        1
    } else {
        0
    }
}

/// Creates an executor that executes an instruction of type R.
/// PERF: if `op: F` where `F: Fn`, this is apparently inlined correctly, so performance is not
/// affected. However, if `op: fn(u32, u32) -> R`, performance is significantly worse! Like,
/// 50%-60% worse. This might be because generics are monomorphized, and each lambda is a different
/// Fn type.
#[inline(always)]
fn exec_type_r<R, F>(rd: u8, rs1: u8, rs2: u8, op: F) -> Executor
where
    R: IntoRegister,
    F: Fn(u32, u32) -> R + 'static,
{
    Executor::new(move |sim, code| {
        sim.set_reg(rd, op(sim.reg(rs1), sim.reg(rs2)));
        next(sim, code, sim.pc + 4);
    })
}

/// Creates an executor that executes an instruction of type I -- imm. See perf note on
/// [`exec_type_r`]
#[inline(always)]
fn exec_type_i_imm<R, F>(rd: u8, rs1: u8, imm: u32, op: F) -> Executor
where
    R: IntoRegister,
    F: Fn(u32, u32) -> R + 'static,
{
    Executor::new(move |sim, code| {
        sim.set_reg(rd, op(sim.reg(rs1), imm));
        next(sim, code, sim.pc + 4);
    })
}

/// Creates an executor that executes a branch instruction. See perf note on [`exec_type_r`]
#[inline(always)]
fn exec_branch<F>(rs1: u8, rs2: u8, label: usize, op: F) -> Executor
where
    F: Fn(u32, u32) -> bool + 'static,
{
    Executor::new(move |sim, code| {
        let new_pc = if op(sim.reg(rs1), sim.reg(rs2)) {
            label
        } else {
            sim.pc + 4
        };
        next(sim, code, new_pc);
    })
}

/// Compiles all instructions in a slice
pub fn compile_all(i: &[Instruction]) -> Vec<Executor> {
    i.iter().map(compile).collect()
}

/// Compiles a parsed instruction into an executor  
pub fn compile(i: &Instruction) -> Executor {
    use Instruction::*;

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
        Mulh(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| {
            ((a as i32 as i64).wrapping_mul(b as i32 as i64) >> 32) as u32
        }),
        Mulhsu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| {
            // signed rs1 × unsigned rs2
            ((a as i32 as i64).wrapping_mul(b as i64) >> 32) as u32
        }),
        Mulhu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| {
            ((a as u64).wrapping_mul(b as u64) >> 32) as u32
        }),
        Div(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| {
            if b == 0 {
                -1
            } else {
                (a as i32).wrapping_div(b as i32)
            }
        }),
        Divu(rd, rs1, rs2) => {
            exec_type_r(rd, rs1, rs2, |a, b| if b == 0 { u32::MAX } else { a / b })
        }
        Rem(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| {
            if b == 0 {
                a as i32
            } else {
                (a as i32).wrapping_rem(b as i32)
            }
        }),
        Remu(rd, rs1, rs2) => exec_type_r(rd, rs1, rs2, |a, b| if b == 0 { a } else { a % b }),
        URet => Executor::new(move |sim, code| {
            use crate::parser::register_names::UEPC_INDEX;
            next(sim, code, sim.status[UEPC_INDEX as usize] as usize);
        }),

        // Type I
        Ecall => Executor::new(move |sim, code| {
            use EcallSignal::*;
            match sim.ecall() {
                Exit => {} // don't execute the next instruction
                Continue => next(sim, code, sim.pc),
                Nothing => next(sim, code, sim.pc + 4),
            }
        }),
        Ebreak => Executor::new(move |sim, _code| {
            let ctx = &sim.code_ctx[sim.pc / 4];
            eprintln!(
                "   {} has not yet been implemented in FPGRARS!\n{}",
                "ebreak".on_bright_magenta(),
                ctx
            );
            std::process::exit(1);
        }),

        // Type I -- Immediate
        Addi(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a.wrapping_add(b)),
        Slti(rd, rs1, imm) => {
            exec_type_i_imm(rd, rs1, imm, |a, b| from_bool((a as i32) < (b as i32)))
        }
        Sltiu(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| from_bool(a < b)),
        Xori(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a ^ b),
        Ori(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a | b),
        Andi(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a & b),
        Slli(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a << (b & 0x1f)),
        Srli(rd, rs1, imm) => exec_type_i_imm(rd, rs1, imm, |a, b| a >> (b & 0x1f)),
        Srai(rd, rs1, imm) => {
            exec_type_i_imm(rd, rs1, imm, |a, b| (a as i32) >> ((b as i32) & 0x1f))
        }

        // Type I -- Loads
        Lb(rd, imm, rs1) => Executor::new(move |sim, code| {
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_byte(addr) as i8 as u32; // sign-extends
            sim.set_reg(rd, data);
            next(sim, code, sim.pc + 4);
        }),
        Lbu(rd, imm, rs1) => Executor::new(move |sim, code| {
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_byte(addr) as u32;
            sim.set_reg(rd, data);
            next(sim, code, sim.pc + 4);
        }),
        Lh(rd, imm, rs1) => Executor::new(move |sim, code| {
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_half(addr) as i16 as u32; // sign-extends
            sim.set_reg(rd, data);
            next(sim, code, sim.pc + 4);
        }),
        Lhu(rd, imm, rs1) => Executor::new(move |sim, code| {
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_half(addr) as u32;
            sim.set_reg(rd, data);
            next(sim, code, sim.pc + 4);
        }),
        Lw(rd, imm, rs1) => Executor::new(move |sim, code| {
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_word(addr);
            sim.set_reg(rd, data);
            next(sim, code, sim.pc + 4);
        }),

        // Type S
        Sb(rs2, imm, rs1) => Executor::new(move |sim, code| {
            sim.memory.set_byte(
                (sim.reg::<u32>(rs1).wrapping_add(imm)) as usize,
                sim.reg::<u8>(rs2),
            );
            next(sim, code, sim.pc + 4);
        }),
        Sh(rs2, imm, rs1) => Executor::new(move |sim, code| {
            sim.memory.set_half(
                (sim.reg::<u32>(rs1).wrapping_add(imm)) as usize,
                sim.reg::<u16>(rs2),
            );
            next(sim, code, sim.pc + 4);
        }),
        Sw(rs2, imm, rs1) => Executor::new(move |sim, code| {
            sim.memory.set_word(
                (sim.reg::<u32>(rs1).wrapping_add(imm)) as usize,
                sim.reg::<u32>(rs2),
            );
            next(sim, code, sim.pc + 4);
        }),

        // Type B
        Beq(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| a == b),
        Bne(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| a != b),
        Blt(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| (a as i32) < (b as i32)),
        Bge(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| (a as i32) >= (b as i32)),
        Bltu(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| a < b),
        Bgeu(rs1, rs2, label) => exec_branch(rs1, rs2, label, |a, b| a >= b),

        // Jumps
        Jalr(rd, rs1, imm) => Executor::new(move |sim, code| {
            // This produces a weird result for `jalr s0 s0 0`. s0 is set to pc+4 before the jump occurs
            // so it works as a nop. Maybe this is correct, maybe it's not, but I'll copy the behavior seen in
            // RARS to be consistent.
            sim.set_reg(rd, (sim.pc + 4) as u32);
            let new_pc = (sim.reg::<i32>(rs1) + (imm as i32)) as usize & !1;
            next(sim, code, new_pc);
        }),
        Jal(rd, label) => Executor::new(move |sim, code| {
            sim.set_reg(rd, (sim.pc + 4) as u32);
            next(sim, code, label);
        }),

        // CSR
        CsrRw(rd, fcsr, rs1) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] = sim.reg::<u32>(rs1);
            next(sim, code, sim.pc + 4);
        }),
        CsrRwi(rd, fcsr, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] = imm;
            next(sim, code, sim.pc + 4);
        }),
        CsrRs(rd, fcsr, rs1) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] |= sim.reg::<u32>(rs1);
            next(sim, code, sim.pc + 4);
        }),
        CsrRsi(rd, fcsr, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] |= imm;
            next(sim, code, sim.pc + 4);
        }),
        CsrRc(rd, fcsr, rs1) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] &= !sim.reg::<u32>(rs1);
            next(sim, code, sim.pc + 4);
        }),
        CsrRci(rd, fcsr, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.get_status(fcsr));
            sim.status[fcsr as usize] &= !imm;
            next(sim, code, sim.pc + 4);
        }),

        // Type U
        Lui(rd, imm) => {
            let imm = imm << 12;
            Executor::new(move |sim, code| {
                sim.set_reg(rd, imm);
                next(sim, code, sim.pc + 4);
            })
        }
        AuiPc(rd, imm) => {
            let imm = imm << 12;
            Executor::new(move |sim, code| {
                sim.set_reg(rd, sim.pc as u32 + imm);
                next(sim, code, sim.pc + 4);
            })
        }

        // Floats
        Float(ref finstr) => compile_float_instruction(finstr),

        // Pseudoinstructions
        Li(rd, imm) => Executor::new(move |sim, code| {
            sim.set_reg(rd, imm as i32);
            next(sim, code, sim.pc + 4);
        }),

        Mv(rd, rs1) => Executor::new(move |sim, code| {
            sim.set_reg(rd, sim.reg::<u32>(rs1));
            next(sim, code, sim.pc + 4);
        }),
    }
}

/// Compiles a float instruction into an executor.
pub fn compile_float_instruction(i: &FloatInstruction) -> Executor {
    use FloatInstruction::*;

    match *i {
        Add(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1] + sim.floats[rs2];
            next(sim, code, sim.pc + 4);
        }),
        Sub(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1] - sim.floats[rs2];
            next(sim, code, sim.pc + 4);
        }),
        Mul(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1] * sim.floats[rs2];
            next(sim, code, sim.pc + 4);
        }),
        Div(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1] / sim.floats[rs2];
            next(sim, code, sim.pc + 4);
        }),
        Equ(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rs1, rs2) = (rs1 as usize, rs2 as usize);
            sim.set_reg(rd, from_bool(sim.floats[rs1] == sim.floats[rs2]));
            next(sim, code, sim.pc + 4);
        }),
        Le(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rs1, rs2) = (rs1 as usize, rs2 as usize);
            sim.set_reg(rd, from_bool(sim.floats[rs1] <= sim.floats[rs2]));
            next(sim, code, sim.pc + 4);
        }),
        Lt(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rs1, rs2) = (rs1 as usize, rs2 as usize);
            sim.set_reg(rd, from_bool(sim.floats[rs1] < sim.floats[rs2]));
            next(sim, code, sim.pc + 4);
        }),
        Max(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1].max(sim.floats[rs2]);
            next(sim, code, sim.pc + 4);
        }),
        Min(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1].min(sim.floats[rs2]);
            next(sim, code, sim.pc + 4);
        }),
        SgnjS(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1].copysign(sim.floats[rs2]);
            next(sim, code, sim.pc + 4);
        }),
        SgnjNS(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            sim.floats[rd] = sim.floats[rs1].copysign(-sim.floats[rs2]);
            next(sim, code, sim.pc + 4);
        }),
        SgnjXS(rd, rs1, rs2) => Executor::new(move |sim, code| {
            let (rd, rs1, rs2) = (rd as usize, rs1 as usize, rs2 as usize);
            let (a, b) = (sim.floats[rs1], sim.floats[rs2]);

            // I'm pretty sure this is correct (for most architectures anyway)
            // TODO: is it correct?
            sim.floats[rd] = f32::from_bits(a.to_bits() ^ (b.to_bits() & (1 << 31)));

            next(sim, code, sim.pc + 4);
        }),

        // I didn't even know this existed before this project
        Class(rd, rs1) => Executor::new(move |sim, code| {
            let rs1 = rs1 as usize;
            sim.set_reg(rd, class_mask(sim.floats[rs1]));
            next(sim, code, sim.pc + 4);
        }),

        CvtSW(rd, rs1) => Executor::new(move |sim, code| {
            let rd = rd as usize;
            sim.floats[rd] = sim.reg::<i32>(rs1) as f32;
            next(sim, code, sim.pc + 4);
        }),
        CvtSWu(rd, rs1) => Executor::new(move |sim, code| {
            let rd = rd as usize;
            sim.floats[rd] = sim.reg::<u32>(rs1) as f32;
            next(sim, code, sim.pc + 4);
        }),
        CvtWS(rd, rs1) => Executor::new(move |sim, code| {
            let rs1 = rs1 as usize;
            sim.set_reg(rd, sim.floats[rs1] as i32);
            next(sim, code, sim.pc + 4);
        }),
        CvtWuS(rd, rs1) => Executor::new(move |sim, code| {
            let rs1 = rs1 as usize;
            sim.set_reg(rd, sim.floats[rs1] as u32);
            next(sim, code, sim.pc + 4);
        }),

        MvSX(rd, rs1) => Executor::new(move |sim, code| {
            let rd = rd as usize;
            sim.floats[rd] = f32::from_bits(sim.reg::<u32>(rs1));
            next(sim, code, sim.pc + 4);
        }),
        MvXS(rd, rs1) => Executor::new(move |sim, code| {
            let rs1 = rs1 as usize;
            sim.set_reg(rd, sim.floats[rs1].to_bits());
            next(sim, code, sim.pc + 4);
        }),
        Sqrt(rd, rs1) => Executor::new(move |sim, code| {
            let (rd, rs1) = (rd as usize, rs1 as usize);
            sim.floats[rd] = sim.floats[rs1].sqrt();
            next(sim, code, sim.pc + 4);
        }),
        Lw(rd, imm, rs1) => Executor::new(move |sim, code| {
            let rd = rd as usize;
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            let data = sim.memory.get_float(addr);
            sim.floats[rd] = data;
            next(sim, code, sim.pc + 4);
        }),
        Sw(rs2, imm, rs1) => Executor::new(move |sim, code| {
            let x = sim.floats[rs2 as usize];
            let addr = sim.reg::<u32>(rs1).wrapping_add(imm) as usize;
            sim.memory.set_float(addr, x);
            next(sim, code, sim.pc + 4);
        }),
    }
}
