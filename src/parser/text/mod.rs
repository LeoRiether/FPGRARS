use super::{error::ParserError, token::Token, ParserContext};

pub fn parse_instruction(
    tokens: &mut impl Iterator<Item = Token>,
    ctx: &mut ParserContext,
    id: String,
) -> Result<(), ParserError> {
    Ok(())
}
