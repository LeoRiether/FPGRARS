use super::{
    error::{Contextualize, Error, ParserError},
    token::{self, Token},
    ParserContext,
};
use crate::{inner_bail, parser::LabelUseType};
use lazy_static::lazy_static;
use owo_colors::OwoColorize;

lazy_static! {
    static ref TIP_IMMEDIATE: String = format!(
        "Some immediate values are: {}, {}, {:x}, {:b} and '{}'",
        10.bright_blue(),
        (-10).bright_blue(),
        0xFFAABBCC_u32.bright_blue(),
        0b101.bright_blue(),
        'a'.bright_blue()
    );
}

pub fn parse_instruction(
    tokens: &mut impl Iterator<Item = Result<Token, Error>>,
    parser: &mut ParserContext,
    instruction: String,
    instr_ctx: token::Context,
) -> Result<(), Error> {
    let instr = instruction.to_lowercase();
    let instr = instr.as_str();

    let mut ipc = InstructionParsingContext::new(tokens, parser, instr, instr_ctx.clone());

    let found = ipc.parse_type_r()? || ipc.parse_type_i()? || ipc.parse_type_s()?;
    if !found {
        let err = ParserError::UnknownInstruction(instruction).with_context(instr_ctx);
        return Err(err);
    }
    Ok(())
}

struct InstructionParsingContext<'a, TI: Iterator<Item = Result<Token, Error>>> {
    tokens: &'a mut TI,
    parser: &'a mut ParserContext,
    instr: &'a str,
    instr_ctx: token::Context,
}

impl<'a, TI> InstructionParsingContext<'a, TI>
where
    TI: Iterator<Item = Result<Token, Error>>,
{
    fn new(
        tokens: &'a mut TI,
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

    fn immediate(&mut self) -> Result<u32, Error> {
        let token = inner_bail!(self.tokens.next());

        use token::Data::Identifier;
        match token.as_ref().map(|t| (&t.data, t.data.extract_u32())) {
            Some((Identifier(label), _)) => {
                // The immediate is a label
                let x = self.parser.use_label(&label, LabelUseType::Code);
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
        use super::Instruction::{self, *};

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
        use super::Instruction::{self, *};

        #[rustfmt::skip]
        macro_rules! reg { () => { self.register()? }; }
        #[rustfmt::skip]
        macro_rules! imm { () => { self.immediate()? }; }
        let instr: Option<Instruction> = match self.instr {
            "ecall" => Ecall.into(),
            "ebreak" => Ebreak.into(),
            "lb" => Lb(reg!(), imm!(), reg!()).into(),
            "lh" => Lh(reg!(), imm!(), reg!()).into(),
            "lw" => Lw(reg!(), imm!(), reg!()).into(),
            "lbu" => Lbu(reg!(), imm!(), reg!()).into(),
            "lhu" => Lhu(reg!(), imm!(), reg!()).into(),
            "addi" => Addi(reg!(), reg!(), imm!()).into(),
            "slti" => Slti(reg!(), reg!(), imm!()).into(),
            "sltiu" => Sltiu(reg!(), reg!(), imm!()).into(),
            "slli" => Slli(reg!(), reg!(), imm!()).into(),
            "srli" => Srli(reg!(), reg!(), imm!()).into(),
            "srai" => Srai(reg!(), reg!(), imm!()).into(),
            "ori" => Ori(reg!(), reg!(), imm!()).into(),
            "andi" => Andi(reg!(), reg!(), imm!()).into(),
            "xori" => Xori(reg!(), reg!(), imm!()).into(),
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

    fn parse_type_s(&mut self) -> Result<bool, Error> {
        use super::Instruction::{self, *};
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
        let instr: Option<Instruction> = match self.instr {
            "sb" => Sb(reg!(), imm!(), paren!(reg!())).into(),
            "sh" => Sh(reg!(), imm!(), paren!(reg!())).into(),
            "sw" => Sw(reg!(), imm!(), paren!(reg!())).into(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;

    #[test]
    fn test_type_r() {
        let input = "add x1, x2, x3
            sub t0, t1, t2
            sll a0, a1, a2
            sltu s0, s1, s2
            xor t3, t4, t5
            divu s11, s10, s9";
        let mut tokens = Lexer::from_content(String::from(input), "type_r.s");
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
        let mut tokens = Lexer::from_content(String::from(input), "type_s.s");
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
