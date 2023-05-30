pub mod unlabel;
use std::iter::Peekable;

pub use unlabel::unlabel;

use super::{
    error::{Contextualize, Error, ParserError},
    register_names::RegMap,
    token::{self, Token},
    ParserContext,
};
use crate::{inner_bail, instruction::Instruction, parser::LabelUseType};
use lazy_static::lazy_static;
use owo_colors::OwoColorize;

lazy_static! {
    static ref TIP_IMMEDIATE: String = format!(
        "Some immediate values are: {}, {}, {}{:x}, {}{:b} and {}{}{}",
        10.bright_blue(),
        (-10).bright_blue(),
        "0x".bright_blue(),
        0xFFAABBCC_u32.bright_blue(),
        "0b".bright_blue(),
        0b101.bright_blue(),
        "'".bright_blue(),
        'a'.bright_blue(),
        "'".bright_blue(),
    );
}

fn is_register(regs: &RegMap, token: Option<&Result<Token, Error>>) -> bool {
    use token::Data::Identifier;
    match token {
        Some(Ok(Token {
            data: Identifier(id),
            ..
        })) => regs.contains_key(id),
        _ => false,
    }
}

pub fn parse_instruction(
    tokens: &mut Peekable<impl Iterator<Item = Result<Token, Error>>>,
    parser: &mut ParserContext,
    instruction: String,
    instr_ctx: token::Context,
) -> Result<(), Error> {
    let instr = instruction.to_lowercase();
    let instr = instr.as_str();

    let mut ipc = InstructionParsingContext::new(tokens, parser, instr, instr_ctx.clone());

    let found = ipc.parse_type_r()?
        || ipc.parse_type_i()?
        || ipc.parse_type_s()?
        || ipc.parse_type_b_and_jumps()?
        || ipc.parse_csr()?
        || ipc.parse_float()?;
    if !found {
        let err = ParserError::UnknownInstruction(instruction).with_context(instr_ctx);
        return Err(err);
    }
    Ok(())
}

struct InstructionParsingContext<'a, TI: Iterator<Item = Result<Token, Error>>> {
    tokens: &'a mut Peekable<TI>,
    parser: &'a mut ParserContext,
    instr: &'a str,
    instr_ctx: token::Context,
}

impl<'a, TI> InstructionParsingContext<'a, TI>
where
    TI: Iterator<Item = Result<Token, Error>>,
{
    fn new(
        tokens: &'a mut Peekable<TI>,
        parser: &'a mut ParserContext,
        instr: &'a str,
        instr_ctx: token::Context,
    ) -> Self {
        Self {
            tokens,
            parser,
            instr,
            instr_ctx,
        }
    }

    fn register(&mut self) -> Result<u8, Error> {
        let token = inner_bail!(self.tokens.next());
        let regs = &self.parser.regnames.regs;

        use token::Data::Identifier;
        match token.as_ref().map(|t| &t.data) {
            Some(Identifier(id)) if regs.contains_key(id) => Ok(regs[id]),

            None => Err(ParserError::ExpectedRegister(None).with_context(self.instr_ctx.clone())),
            Some(other) => {
                let ctx = token.as_ref().unwrap().ctx.clone();
                Err(ParserError::ExpectedRegister(Some(other.to_string())).with_context(ctx))
            }
        }
    }

    fn status_register(&mut self) -> Result<u8, Error> {
        let token = inner_bail!(self.tokens.next());
        let status = &self.parser.regnames.status;

        use token::Data::Identifier;
        match token.as_ref().map(|t| &t.data) {
            Some(Identifier(id)) if status.contains_key(id) => Ok(status[id]),

            None => {
                Err(ParserError::ExpectedStatusRegister(None).with_context(self.instr_ctx.clone()))
            }
            Some(other) => {
                let ctx = token.as_ref().unwrap().ctx.clone();
                Err(ParserError::ExpectedStatusRegister(Some(other.to_string())).with_context(ctx))
            }
        }
    }

    fn float_register(&mut self) -> Result<u8, Error> {
        let token = inner_bail!(self.tokens.next());
        let floats = &self.parser.regnames.floats;

        use token::Data::Identifier;
        match token.as_ref().map(|t| &t.data) {
            Some(Identifier(id)) if floats.contains_key(id) => Ok(floats[id]),

            None => {
                Err(ParserError::ExpectedFloatRegister(None).with_context(self.instr_ctx.clone()))
            }
            Some(other) => {
                let ctx = token.as_ref().unwrap().ctx.clone();
                Err(ParserError::ExpectedFloatRegister(Some(other.to_string())).with_context(ctx))
            }
        }
    }

    fn immediate(&mut self) -> Result<u32, Error> {
        let token = inner_bail!(self.tokens.next());
        self.immediate_from(token)
    }

    fn immediate_from(&mut self, token: Option<Token>) -> Result<u32, Error> {
        use token::Data::Identifier;
        match token.as_ref().map(|t| (&t.data, t.data.extract_u32())) {
            Some((Identifier(label), _)) => {
                // The immediate is a label
                let ctx = token.as_ref().unwrap().ctx.clone();
                let x = self.parser.use_label(label, LabelUseType::Code, ctx);
                Ok(x)
            }

            Some((_, Some(x))) => {
                // The immediate is a number
                Ok(x)
            }

            None => Err(ParserError::ExpectedImmediate(None)
                .with_context(self.instr_ctx.clone())
                .with_tip(&*TIP_IMMEDIATE)),
            Some((other, _)) => {
                let ctx = token.as_ref().unwrap().ctx.clone();
                Err(ParserError::ExpectedImmediate(Some(other.to_string()))
                    .with_context(ctx)
                    .with_tip(&*TIP_IMMEDIATE))
            }
        }
    }

    fn the_token(&mut self, data: token::Data) -> Result<token::Data, Error> {
        let token = inner_bail!(self.tokens.next());

        match token.as_ref().map(|t| &t.data) {
            Some(d) if &data == d => Ok(data),

            None => {
                Err(ParserError::ExpectedToken(data, None).with_context(self.instr_ctx.clone()))
            }
            Some(other) => {
                let ctx = token.as_ref().unwrap().ctx.clone();
                Err(ParserError::ExpectedToken(data, Some(other.clone())).with_context(ctx))
            }
        }
    }

    fn parse_type_r(&mut self) -> Result<bool, Error> {
        use super::Instruction::*;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        let instr: Option<Instruction> = match self.instr {
            "add" => Add(reg!(), reg!(), reg!()).into(),
            "sub" => Sub(reg!(), reg!(), reg!()).into(),
            "sll" => Sll(reg!(), reg!(), reg!()).into(),
            "slt" => Slt(reg!(), reg!(), reg!()).into(),
            "sltu" => Sltu(reg!(), reg!(), reg!()).into(),
            "xor" => Xor(reg!(), reg!(), reg!()).into(),
            "srl" => Srl(reg!(), reg!(), reg!()).into(),
            "sra" => Sra(reg!(), reg!(), reg!()).into(),
            "or" => Or(reg!(), reg!(), reg!()).into(),
            "and" => And(reg!(), reg!(), reg!()).into(),
            "mul" => Mul(reg!(), reg!(), reg!()).into(),
            "div" => Div(reg!(), reg!(), reg!()).into(),
            "divu" => Divu(reg!(), reg!(), reg!()).into(),
            "rem" => Rem(reg!(), reg!(), reg!()).into(),
            "remu" => Remu(reg!(), reg!(), reg!()).into(),
            "uret" => URet.into(),
            "neg" => Sub(reg!(), 0, reg!()).into(),
            "not" => Xori(reg!(), reg!(), (-1i32) as u32).into(),
            "mv" => Mv(reg!(), reg!()).into(),
            "snez" => Sltu(reg!(), 0, reg!()).into(),
            "sltz" => Slt(reg!(), reg!(), 0).into(),
            "sgtz" => Slt(reg!(), 0, reg!()).into(),
            _ => None,
        };

        match instr {
            Some(instr) => {
                self.parser.code.push(instr);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn parse_type_i(&mut self) -> Result<bool, Error> {
        use super::Instruction::*;
        use token::Data::Char;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }
        macro_rules! paren {
            ($inner:expr) => {{
                self.the_token(Char('('))?;
                let res = $inner;
                self.the_token(Char(')'))?;
                res
            }};
        }

        // TODO: improve error messages for this
        macro_rules! load_madness {
            ($instruction:expr) => {{
                let rd = reg!();
                if let Some(Ok(Token {
                    data: Char('('), ..
                })) = self.tokens.peek()
                {
                    // lw rd, (rs1)
                    let rs1 = paren!(reg!());
                    $instruction(rd, 0, rs1)
                } else {
                    let imm = imm!();
                    if let Some(Ok(Token {
                        data: Char('('), ..
                    })) = self.tokens.peek()
                    {
                        // lw rd, imm(rs1)
                        let rs1 = paren!(reg!());
                        $instruction(rd, imm, rs1)
                    } else {
                        // lw rd, label
                        // gets transformed to:
                        // la rd label
                        // lw rd, 0(rd)
                        self.parser.code.push(Li(rd, imm));
                        self.parser.code.push($instruction(rd, 0, rd));
                        return Ok(true);
                    }
                }
            }};
        }

        let instr = match self.instr {
            "ecall" => Ecall,
            "ebreak" => Ebreak,
            "lb" => load_madness!(Lb),
            "lh" => load_madness!(Lh),
            "lw" => load_madness!(Lw),
            "lbu" => Lbu(reg!(), imm!(), paren!(reg!())),
            "lhu" => Lhu(reg!(), imm!(), paren!(reg!())),
            "addi" => Addi(reg!(), reg!(), imm!()),
            "slti" => Slti(reg!(), reg!(), imm!()),
            "sltiu" => Sltiu(reg!(), reg!(), imm!()),
            "slli" => Slli(reg!(), reg!(), imm!()),
            "srli" => Srli(reg!(), reg!(), imm!()),
            "srai" => Srai(reg!(), reg!(), imm!()),
            "ori" => Ori(reg!(), reg!(), imm!()),
            "andi" => Andi(reg!(), reg!(), imm!()),
            "xori" => Xori(reg!(), reg!(), imm!()),
            "seqz" => Sltiu(reg!(), reg!(), 1),
            "lui" => Li(reg!(), imm!() << 12),
            "li" | "la" => Li(reg!(), imm!()),
            "nop" => Addi(0, 0, 0),
            _ => return Ok(false),
        };
        self.parser.code.push(instr);
        Ok(true)
    }

    fn parse_type_s(&mut self) -> Result<bool, Error> {
        use super::Instruction::*;
        use token::Data::Char;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }
        macro_rules! paren {
            ($inner:expr) => {{
                self.the_token(Char('('))?;
                let res = $inner;
                self.the_token(Char(')'))?;
                res
            }};
        }
        let instr = match self.instr {
            "sb" => Sb(reg!(), imm!(), paren!(reg!())),
            "sh" => Sh(reg!(), imm!(), paren!(reg!())),
            "sw" => Sw(reg!(), imm!(), paren!(reg!())),
            "ret" => Jalr(0, 1, 0),
            _ => return Ok(false),
        };

        self.parser.code.push(instr);
        Ok(true)
    }

    fn parse_type_b_and_jumps(&mut self) -> Result<bool, Error> {
        use super::Instruction::*;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }
        let instr = match self.instr {
            "beq" => Beq(reg!(), reg!(), imm!() as usize),
            "bne" => Bne(reg!(), reg!(), imm!() as usize),
            "blt" => Blt(reg!(), reg!(), imm!() as usize),
            "bge" => Bge(reg!(), reg!(), imm!() as usize),
            "bltu" => Bltu(reg!(), reg!(), imm!() as usize),
            "bgeu" => Bgeu(reg!(), reg!(), imm!() as usize),
            "beqz" => Beq(reg!(), 0, imm!() as usize),
            "bnez" => Bne(reg!(), 0, imm!() as usize),
            "bltz" => Blt(reg!(), 0, imm!() as usize),
            "bgez" => Bge(reg!(), 0, imm!() as usize),
            "bltuz" => Bltu(reg!(), 0, imm!() as usize),
            "bgeuz" => Bgeu(reg!(), 0, imm!() as usize),
            "blez" => Bge(0, reg!(), imm!() as usize),
            "bgtz" => Blt(0, reg!(), imm!() as usize),
            "ble" => {
                let (r1, r2) = (reg!(), reg!());
                Bge(r2, r1, imm!() as usize)
            }
            "bgt" => {
                let (r1, r2) = (reg!(), reg!());
                Blt(r2, r1, imm!() as usize)
            }
            "bleu" => {
                let (r1, r2) = (reg!(), reg!());
                Bgeu(r2, r1, imm!() as usize)
            }
            "bgtu" => {
                let (r1, r2) = (reg!(), reg!());
                Bltu(r2, r1, imm!() as usize)
            }
            "jal" if is_register(&self.parser.regnames.regs, self.tokens.peek()) => {
                Jal(reg!(), imm!() as usize)
            }
            "jal" => Jal(1, imm!() as usize),
            "jalr" => Jalr(reg!(), reg!(), imm!()),
            "jr" => Jalr(reg!(), 0, 0),
            "call" => Jal(1, imm!() as usize),
            "j" | "tail" | "b" => Jal(0, imm!() as usize),
            _ => return Ok(false),
        };

        self.parser.code.push(instr);
        Ok(true)
    }

    fn parse_csr(&mut self) -> Result<bool, Error> {
        use super::Instruction::*;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! fcsr { () => { self.status_register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }

        macro_rules! csr_small {
            ($inst:expr) => {{
                let (rs1, fcsr) = (reg!(), fcsr!());
                $inst(0, fcsr, rs1)
            }};
        }

        let instr = match self.instr {
            "csrw" => csr_small!(CsrRw),
            "csrc" => csr_small!(CsrRc),
            "csrs" => csr_small!(CsrRs),
            "csrwi" => CsrRwi(0, fcsr!(), imm!()),
            "csrci" => CsrRci(0, fcsr!(), imm!()),
            "csrsi" => CsrRsi(0, fcsr!(), imm!()),
            "csrrs" => CsrRs(reg!(), fcsr!(), reg!()),
            "csrrw" => CsrRw(reg!(), fcsr!(), reg!()),
            "csrrc" => CsrRc(reg!(), fcsr!(), reg!()),
            "csrrsi" => CsrRsi(reg!(), fcsr!(), imm!()),
            "csrrwi" => CsrRwi(reg!(), fcsr!(), imm!()),
            "csrrci" => CsrRci(reg!(), fcsr!(), imm!()),
            "csrr" => CsrRs(reg!(), fcsr!(), 0),
            _ => return Ok(false),
        };

        self.parser.code.push(instr);
        Ok(true)
    }

    fn parse_float(&mut self) -> Result<bool, Error> {
        use super::FloatInstruction as F;
        use token::Data::Char;

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }
        #[rustfmt::skip]
        macro_rules! freg { () => { self.float_register()? }; }
        macro_rules! paren {
            ($inner:expr) => {{
                self.the_token(Char('('))?;
                let res = $inner;
                self.the_token(Char(')'))?;
                res
            }};
        }

        let instr = match self.instr {
            "fadd.s" => F::Add(freg!(), freg!(), freg!()),
            "fsub.s" => F::Sub(freg!(), freg!(), freg!()),
            "fmul.s" => F::Mul(freg!(), freg!(), freg!()),
            "fdiv.s" => F::Div(freg!(), freg!(), freg!()),
            "feq.s" => F::Equ(reg!(), freg!(), freg!()),
            "fle.s" => F::Le(reg!(), freg!(), freg!()),
            "flt.s" => F::Lt(reg!(), freg!(), freg!()),
            "fmax.s" => F::Max(freg!(), freg!(), freg!()),
            "fmin.s" => F::Min(freg!(), freg!(), freg!()),
            "fsgnj.s" => F::SgnjS(freg!(), freg!(), freg!()),
            "fsgnjn.s" => F::SgnjNS(freg!(), freg!(), freg!()),
            "fsgnjx.s" => F::SgnjXS(freg!(), freg!(), freg!()),
            "fclass.s" => F::Class(reg!(), freg!()),
            "fcvt.s.w" => F::CvtSW(freg!(), reg!()),
            "fcvt.s.wu" => F::CvtSWu(freg!(), reg!()),
            "fcvt.w.s" => F::CvtWS(reg!(), freg!()),
            "fcvt.wu.s" => F::CvtWuS(reg!(), freg!()),
            "fmv.s.x" => F::MvSX(freg!(), reg!()),
            "fmv.x.s" => F::MvXS(reg!(), freg!()),
            "fsqrt.s" => F::Sqrt(freg!(), freg!()),
            "fabs.s" => {
                let (rd, rs1) = (freg!(), freg!());
                F::SgnjXS(rd, rs1, rs1)
            }
            "fmv.s" => {
                let (rd, rs1) = (freg!(), freg!());
                F::SgnjNS(rd, rs1, rs1)
            }
            "fneg.s" => {
                let (rd, rs1) = (freg!(), freg!());
                F::SgnjNS(rd, rs1, rs1)
            }
            "flw" => F::Lw(freg!(), imm!(), paren!(reg!())),
            "fsw" => F::Sw(freg!(), imm!(), paren!(reg!())),
            _ => return Ok(false),
        };

        self.parser.code.push(Instruction::Float(instr));
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;

    #[test]
    fn test_la() {
        let input = "la x10 0x800";
        let mut tokens = Lexer::from_content(String::from(input), "type_r.s").peekable();
        let mut parser = ParserContext::default();

        let instruction = tokens.next().unwrap().unwrap().data.to_string();
        let res = parse_instruction(
            &mut tokens,
            &mut parser,
            instruction,
            token::Context::empty(),
        );
        assert!(res.is_ok());

        use super::super::Instruction::*;
        assert_eq!(&parser.code, &[Li(10, 0x800)])
    }

    #[test]
    fn test_type_r() {
        let input = "add x1, x2, x3
            sub t0, t1, t2
            sll a0, a1, a2
            sltu s0, s1, s2
            xor t3, t4, t5
            divu s11, s10, s9";
        let mut tokens = Lexer::from_content(String::from(input), "type_r.s").peekable();
        let mut parser = ParserContext::default();

        for _ in 0..6 {
            let instruction = tokens.next().unwrap().unwrap().data.to_string();
            let res = parse_instruction(
                &mut tokens,
                &mut parser,
                instruction,
                token::Context::empty(),
            );
            assert!(res.is_ok());
        }

        use super::super::Instruction::*;
        assert_eq!(
            &parser.code,
            &[
                Add(1, 2, 3),
                Sub(5, 6, 7),
                Sll(10, 11, 12),
                Sltu(8, 9, 18),
                Xor(28, 29, 30),
                Divu(27, 26, 25),
            ]
        )
    }

    #[test]
    fn test_type_s() {
        let input = "sb x1, 0(x2)
            sh x10, 0xFF(x30)
            sw x0, 'a'(x0)";
        let mut tokens = Lexer::from_content(String::from(input), "type_s.s").peekable();
        let mut parser = ParserContext::default();

        for _ in 0..3 {
            let instruction = tokens.next().unwrap().unwrap().data.to_string();
            let res = parse_instruction(
                &mut tokens,
                &mut parser,
                instruction,
                token::Context::empty(),
            );
            assert!(res.is_ok());
        }

        use super::super::Instruction::*;
        assert_eq!(
            &parser.code,
            &[Sb(1, 0, 2), Sh(10, 0xFF, 30), Sw(0, 'a' as u32, 0),]
        )
    }
}
