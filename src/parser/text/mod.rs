use super::{error::Error, token::Token, ParserContext};

pub fn parse_instruction(
    tokens: &mut impl Iterator<Item = Result<Token, Error>>,
    ctx: &mut ParserContext,
    id: String,
) -> Result<(), Error> {
    Ok(())
}
