//!
//! Parses RISC-V code into code and data parts, so it can be used in the simulator module.
//! We use a lot of mnemonics here, I'll try to link to a cheatsheet here later.
//!

use radix_trie::Trie;

pub mod register_names;
use register_names::{self as reg_names, RegMap};

mod combinators;
use combinators::*;

mod preprocessor;
pub use preprocessor::*;

mod util;
pub use util::*;

/// Floating point instructions.
/// In a separate enum because maybe someday I'll have a cargo feature to disable
/// floating point instructions.
/// Everything here is single precision, no doubles allowed.
#[derive(Debug, PartialEq, Eq)]
pub enum FloatInstruction {
    /// rd, rs1, rs2
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    Equ(u8, u8, u8), // Eq was taken
    Le(u8, u8, u8),
    Lt(u8, u8, u8),
    Max(u8, u8, u8),
    Min(u8, u8, u8),
    SgnjS(u8, u8, u8),
    SgnjNS(u8, u8, u8),
    SgnjXS(u8, u8, u8),

    /// rd, rs1
    Class(u8, u8),
    CvtSW(u8, u8), // fcvt.s.w
    CvtSWu(u8, u8), // fcvt.s.wu
    CvtWS(u8, u8), // fcvt.w.s
    CvtWuS(u8, u8), // fcvw.wu.s
    MvSX(u8, u8), // fmv.s.x
    MvXS(u8, u8), // fmv.x.s
    Sqrt(u8, u8),

    Lw(u8, u32, u8),
    Sw(u8, u32, u8),
}

/// Giant enum that represents a single RISC-V instruction and its arguments
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    // Type R
    /// rd, rs1, rs2
    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Sll(u8, u8, u8),
    Slt(u8, u8, u8),
    Sltu(u8, u8, u8),
    Xor(u8, u8, u8),
    Srl(u8, u8, u8),
    Sra(u8, u8, u8),
    Or(u8, u8, u8),
    And(u8, u8, u8),
    Mul(u8, u8, u8), // TODO: mulh, mulhsu, mulhu
    Div(u8, u8, u8),
    Divu(u8, u8, u8),
    Rem(u8, u8, u8),
    Remu(u8, u8, u8),

    // Type I
    Ecall,
    /// rd, imm, rs1
    Lb(u8, u32, u8),
    Lh(u8, u32, u8),
    Lw(u8, u32, u8),
    Lbu(u8, u32, u8),
    Lhu(u8, u32, u8),
    /// rd, rs1, imm
    Addi(u8, u8, u32),
    Slti(u8, u8, u32),
    Sltiu(u8, u8, u32),
    Slli(u8, u8, u32),
    Srli(u8, u8, u32),
    Srai(u8, u8, u32),
    Ori(u8, u8, u32),
    Andi(u8, u8, u32),
    Xori(u8, u8, u32),

    // Type S
    /// rs2, imm, rs1
    Sb(u8, u32, u8),
    Sh(u8, u32, u8),
    Sw(u8, u32, u8),

    // Type SB + jumps
    /// rs1, rs2, label
    Beq(u8, u8, usize),
    Bne(u8, u8, usize),
    Blt(u8, u8, usize),
    Bge(u8, u8, usize),
    Bltu(u8, u8, usize),
    Bgeu(u8, u8, usize),
    /// rd, rs1, imm
    Jalr(u8, u8, u32),
    /// rd, label
    Jal(u8, usize),

    // CSR
    /// rd, fcsr, rs1
    CsrRw(u8, u8, u8),
    CsrRs(u8, u8, u8),
    CsrRc(u8, u8, u8),
    /// rd, fcsr, imm
    CsrRwi(u8, u8, u32),
    CsrRsi(u8, u8, u32),
    CsrRci(u8, u8, u32),

    // Floating point
    Float(FloatInstruction),

    // Some pseudoinstructions
    /// rd, imm
    Li(u8, u32),
    /// rd, rs1
    Mv(u8, u8),

    Ret,
    URet,
}

/// Also giant enum that represents a single RISC-V instruction, but we save
/// labels as strings because it might not have parsed it yet (for example,
/// consider a jump instruction that jumps to a label in the next line).
///
/// We process the labels stored after the entire file has been parsed.
#[derive(Debug, PartialEq, Eq)]
enum PreLabelInstruction {
    Beq(u8, u8, String),
    Bne(u8, u8, String),
    Blt(u8, u8, String),
    Bge(u8, u8, String),
    Bltu(u8, u8, String),
    Bgeu(u8, u8, String),
    Jal(u8, String),

    /// Gets mapped to an Instruction::Li(rd, position) after unlabeling
    La(u8, String),

    Other(Instruction),
}

impl From<Instruction> for PreLabelInstruction {
    fn from(i: Instruction) -> PreLabelInstruction {
        PreLabelInstruction::Other(i)
    }
}

impl From<FloatInstruction> for PreLabelInstruction {
    fn from(i: FloatInstruction) -> PreLabelInstruction {
        PreLabelInstruction::Other(Instruction::Float(i))
    }
}

/// Represents a successful parser result. This is the same format the simulator
/// will use to execute the instructions
pub struct Parsed {
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
}

pub type ParseResult = Result<Parsed, Error>;

/// The "current" parser directive
enum Directive {
    Text,
    Data,
}

pub trait RISCVParser {
    /// Parses an iterator of preprocessed lines and returns the instructions and
    /// the data it parsed. Remember to preprocess the iterator before calling this,
    /// as `parse_riscv` does not understand macros and includes.
    /// ```
    /// parse::file_lines("riscv.s".to_owned())?
    ///     .parse_includes()
    ///     .parse_macros()
    ///     .parse_riscv(DATA_SIZE)?;
    /// ```
    ///
    /// The `data_segment_size` parameter is the final size of the data segment, in bytes.
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult;
}

type FullRegMap = (RegMap, RegMap, RegMap);

impl<I: Iterator<Item = String>> RISCVParser for I {
    fn parse_riscv(self, data_segment_size: usize) -> ParseResult {
        use combinators::*;

        let regmaps = (reg_names::regs(), reg_names::floats(), reg_names::status());
        let mut labels = Trie::<String, usize>::new();

        let mut directive = Directive::Text;
        let mut code = Vec::new();
        let mut data = Vec::with_capacity(data_segment_size);

        for line in self {
            let line = match parse_label(&line) {
                Ok((rest, label)) => {
                    let label_pos = match directive {
                        Directive::Text => code.len() * 4,
                        Directive::Data => data.len(),
                    };
                    labels.insert(label.to_owned(), label_pos);
                    rest
                }
                Err(_) => &line,
            };

            let (line, _) = separator0(line)?;
            if line.is_empty() {
                continue;
            }

            // Identify directives
            // This accepts stuff like ".textSOMETHING" or ".database", but RARS accepts it too
            // Gotta be consistent! ¯\_(ツ)_/¯
            if line.starts_with(".data") {
                directive = Directive::Data;
                continue;
            } else if line.starts_with(".text") {
                directive = Directive::Text;
                continue;
            }

            match directive {
                Directive::Text => {
                    let instruction = match parse_text(line, &regmaps) {
                        Ok(x) => x,
                        Err(e) => return Err(Error::OnLine(line.to_owned(), Box::new(e))),
                    };
                    code.push(instruction);
                }
                // Directive::Data => unimplemented!("No .data implementation yet"),
                Directive::Data => {},
            }
        }

        let code: Result<Vec<Instruction>, Error> = code
            .into_iter()
            .map(|i| unlabel_instruction(i, &labels))
            .collect();
        let mut code = code?;

        // If the program ever drops off bottom, we make an "exit" ecall and terminate execution
        code.extend(vec![
            Instruction::Li(17, 10), // li a7 10
            Instruction::Ecall,
        ]);

        data.resize(data_segment_size, 0);
        Ok(Parsed { code, data })
    }
}

fn parse_text(s: &str, regmaps: &FullRegMap) -> Result<PreLabelInstruction, Error> {
    let (regs, _floats, status) = regmaps;
    use Instruction::*;
    use PreLabelInstruction as pre;

    let (s, instruction) = one_arg(s)?;

    macro_rules! type_i {
        ($inst:expr) => {
            args_type_i(s, &regs).map(|(rd, rs1, imm)| $inst(rd, rs1, imm).into())?
        };
    }

    macro_rules! type_r {
        ($inst:expr) => {
            args_type_r(s, &regs).map(|(rd, rs1, rs2)| $inst(rd, rs1, rs2).into())?
        };
    }

    macro_rules! type_sb {
        ($inst:expr) => {
            args_type_sb(s, &regs).map(|(rs1, rs2, label)| $inst(rs1, rs2, label))?
        };
    }

    // bgez, bnez, ...
    macro_rules! type_sb_z {
        ($inst:expr) => {
            args_jal(s, &regs).map(|(rs1, label)| $inst(rs1, 0, label))?
        };
    }

    // Reverses the order of rs1 and rs2 to convert, for example,
    // `ble t0 t1 label` into `bge t1 t0 label`
    macro_rules! type_sb_reversed {
        ($inst:expr) => {
            args_type_sb(s, &regs).map(|(rs1, rs2, label)| $inst(rs2, rs1, label))?
        };
    }

    // blez, ...
    macro_rules! type_sb_reversed_z {
        ($inst:expr) => {
            args_jal(s, &regs).map(|(rs1, label)| $inst(0, rs1, label))?
        };
    }

    macro_rules! type_s {
        ($inst:expr) => {
            args_type_s(s, &regs).map(|(r1, imm, r2)| $inst(r1, imm, r2).into())?
        };
    }

    macro_rules! csr {
        ($inst:expr) => {
            args_csr(s, &regs, &status).map(|(rd, fcsr, rs1)| $inst(rd, fcsr, rs1).into())?
        };
    }

    macro_rules! csr_imm {
        ($inst:expr) => {
            args_csr_imm(s, &regs, &status).map(|(rd, fcsr, imm)| $inst(rd, fcsr, imm).into())?
        };
    }

    macro_rules! csr_small {
        ($inst:expr) => {
            args_csr_small(s, &regs, &status).map(|(rs1, fcsr)| $inst(0, fcsr, rs1).into())?
        };
    }

    macro_rules! csr_small_imm {
        ($inst:expr) => {
            args_csr_small_imm(s, &status).map(|(fcsr, imm)| $inst(0, fcsr, imm).into())?
        };
    }

    let parsed = match instruction.to_lowercase().as_str() {
        // Type R
        "add" => type_r!(Add),
        "sub" => type_r!(Sub),
        "sll" => type_r!(Sll),
        "slt" => type_r!(Slt),
        "sltu" => type_r!(Sltu),
        "xor" => type_r!(Xor),
        "srl" => type_r!(Srl),
        "sra" => type_r!(Sra),
        "or" => type_r!(Or),
        "and" => type_r!(And),
        "mul" => type_r!(Mul),
        "div" => type_r!(Div),
        "divu" => type_r!(Divu),
        "rem" => type_r!(Rem),
        "remu" => type_r!(Remu),
        "neg" => args_mv(s, &regs).map(|(rd, rs1)| Sub(rd, 0, rs1).into())?,
        "not" => args_mv(s, &regs).map(|(rd, rs1)| Xori(rd, rs1, (-1i32) as u32).into())?,
        "mv" => args_mv(s, &regs).map(|(rd, rs1)| Mv(rd, rs1).into())?,

        // Type I
        "addi" => type_i!(Addi),
        "slli" => type_i!(Slli),
        "slti" => type_i!(Slti),
        "sltiu" => type_i!(Sltiu),
        "xori" => type_i!(Xori),
        "srli" => type_i!(Srli),
        "srai" => type_i!(Srai),
        "ori" => type_i!(Ori),
        "andi" => type_i!(Andi),
        "jalr" => type_i!(Jalr),
        "jr" => one_reg(&regs)(s).map(|(_i, rs1)| Jalr(0, rs1, 0).into())?,

        // Type I, loads from memory
        "lb" => type_s!(Lb),
        "lh" => type_s!(Lh),
        "lw" => type_s!(Lw),
        "lbu" => type_s!(Lbu),
        "lhu" => type_s!(Lhu),

        // Type S
        "sb" => type_s!(Sb),
        "sh" => type_s!(Sh),
        "sw" => type_s!(Sw),

        // Type SB and pseudoinstructions that map to SBs
        "beq" => type_sb!(pre::Beq),
        "bne" => type_sb!(pre::Bne),
        "blt" => type_sb!(pre::Blt),
        "bge" => type_sb!(pre::Bge),
        "bltu" => type_sb!(pre::Bltu),
        "bgeu" => type_sb!(pre::Bgeu),
        "bgt" => type_sb_reversed!(pre::Blt),
        "ble" => type_sb_reversed!(pre::Bge),
        "bgtu" => type_sb_reversed!(pre::Bltu),
        "bleu" => type_sb_reversed!(pre::Bgeu),
        "beqz" => type_sb_z!(pre::Beq),
        "bnez" => type_sb_z!(pre::Bne),
        "bltz" => type_sb_z!(pre::Blt),
        "bgez" => type_sb_z!(pre::Bge),
        "bltuz" => type_sb_z!(pre::Bltu),
        "bgeuz" => type_sb_z!(pre::Bgeu),
        "bgtz" => type_sb_reversed_z!(pre::Blt),
        "blez" => type_sb_reversed_z!(pre::Bge),

        // CSR
        "csrw" => csr_small!(CsrRw),
        "csrc" => csr_small!(CsrRc),
        "csrs" => csr_small!(CsrRs),
        "csrwi" => csr_small_imm!(CsrRwi),
        "csrci" => csr_small_imm!(CsrRci),
        "csrsi" => csr_small_imm!(CsrRsi),
        "csrrs" => csr!(CsrRs),
        "csrrw" => csr!(CsrRw),
        "csrrc" => csr!(CsrRc),
        "csrrsi" => csr_imm!(CsrRsi),
        "csrrwi" => csr_imm!(CsrRwi),
        "csrrci" => csr_imm!(CsrRci),
        "csrr" => args_csr_small(s, &regs, &status).map(|(rd, fcsr)| CsrRs(rd, fcsr, 0).into())?,

        // Jumps
        "jal" => parse_jal(s, &regs)?,
        "call" => one_arg(s).map(|(_i, label)| pre::Jal(1, label.to_owned()))?,
        "j" | "tail" | "b" => one_arg(s).map(|(_i, label)| pre::Jal(0, label.to_owned()))?,
        "ret" => Ret.into(),

        "ecall" => Ecall.into(),

        // not quite a `jal`, but the same arguments
        "la" => args_jal(s, &regs).map(|(rd, label)| pre::La(rd, label.to_owned()))?,

        "li" => args_li(s, &regs).map(|(rd, imm)| Li(rd, imm).into())?,
        "lui" => args_li(s, &regs).map(|(rd, imm)| Li(rd, imm << 12).into())?,

        "nop" => Mv(0, 0).into(),

        // TODO
        "fadd.s" => Mv(0, 0).into(),
        "fclass.s" => Mv(0, 0).into(),
        "fcvt.s.w" => Mv(0, 0).into(),
        "fcvt.s.wu" => Mv(0, 0).into(),
        "fcvt.w.s" => Mv(0, 0).into(),
        "fcvt.wu.s" => Mv(0, 0).into(),
        "fdiv.s" => Mv(0, 0).into(),
        "feq.s" => Mv(0, 0).into(),
        "fle.s" => Mv(0, 0).into(),
        "flt.s" => Mv(0, 0).into(),
        "flw" => Mv(0, 0).into(),
        "fmax.s" => Mv(0, 0).into(),
        "fmin.s" => Mv(0, 0).into(),
        "fmv.s.x" => Mv(0, 0).into(),
        "fmv.x.s" => Mv(0, 0).into(),
        "fsgnj.s" => Mv(0, 0).into(),
        "fsgnjn.s" => Mv(0, 0).into(),
        "fsgnjx.s" => Mv(0, 0).into(),
        "fsqrt.s" => Mv(0, 0).into(),
        "fsub.s" => Mv(0, 0).into(),
        "fsw" => Mv(0, 0).into(),
        "fabs.s" => Mv(0, 0).into(),
        "fmv.s" => Mv(0, 0).into(),
        "fmul.s" => Mv(0, 0).into(),
        "fneg.s" => Mv(0, 0).into(),

        "uret" => URet.into(),

        dont_know => return Err(Error::InstructionNotFound(dont_know.to_owned())),
    };

    Ok(parsed)
}

/// Parses either `jal rd label` or `jal label`. In the last case, we set `rd = ra`
fn parse_jal<'a>(s: &'a str, regs: &RegMap) -> Result<PreLabelInstruction, Error> {
    use PreLabelInstruction as pre;
    args_jal(s, regs)
        .map(|(rd, label)| pre::Jal(rd, label.to_owned()))
        .or_else(|_| one_arg(s).map(|(_i, label)| pre::Jal(1, label.to_owned())))
        .map_err(|e| e.into())
}

/// Transforms a PreLabelInstruction into a normal Instruction by "commiting" the labels
/// into positions in the code. For example, Jal(0, "Label") maps to Jal(0, labels_trie.get("Label"))
fn unlabel_instruction(
    instruction: PreLabelInstruction,
    labels: &Trie<String, usize>,
) -> Result<Instruction, Error> {
    use Instruction::*;
    use PreLabelInstruction as p;

    macro_rules! unlabel {
        ($inst:ident, $rd:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rd, pos))
                .ok_or(Error::LabelNotFound($label))
        };
        ($inst:ident, $rs1:ident, $rs2:ident, $label:ident) => {
            labels
                .get(&$label)
                .map(|&pos| $inst($rs1, $rs2, pos))
                .ok_or(Error::LabelNotFound($label))
        };
    }

    match instruction {
        p::Jal(rd, label) => unlabel!(Jal, rd, label),
        p::Beq(rs1, rs2, label) => unlabel!(Beq, rs1, rs2, label),
        p::Bne(rs1, rs2, label) => unlabel!(Bne, rs1, rs2, label),
        p::Bge(rs1, rs2, label) => unlabel!(Bge, rs1, rs2, label),
        p::Blt(rs1, rs2, label) => unlabel!(Blt, rs1, rs2, label),
        p::Bltu(rs1, rs2, label) => unlabel!(Bltu, rs1, rs2, label),
        p::Bgeu(rs1, rs2, label) => unlabel!(Bgeu, rs1, rs2, label),
        p::La(rd, label) => labels
            .get(&label)
            .map(|&pos| Li(rd, pos as u32))
            .ok_or(Error::LabelNotFound(label)),
        p::Other(instruction) => Ok(instruction),
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction::*;
    use super::PreLabelInstruction as pre;
    use super::*;

    use lazy_static::*;
    lazy_static! {
        static ref FULLREG: FullRegMap =
            { (reg_names::regs(), reg_names::floats(), reg_names::status()) };
    }

    #[test]
    fn test_parse_text() {
        assert_eq!(
            parse_text("add s0, s0, s1,,,, ", &FULLREG).map_err(|_| ()),
            Ok(Add(8, 8, 9).into())
        );
        assert_eq!(
            parse_text("j label", &FULLREG).map_err(|_| ()),
            Ok(pre::Jal(0, "label".to_owned()).into())
        );
        assert_eq!(
            parse_text("bgtz x1 somewhere", &FULLREG).map_err(|_| ()),
            Ok(pre::Blt(0, 1, "somewhere".to_owned()).into())
        );
    }
}