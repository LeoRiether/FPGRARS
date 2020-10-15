use combine::*;

pub fn remove_comment<Input>() -> impl Parser<Input, Output = String>
    where Input : Stream<Token = char>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    many(satisfy(|c| c != '#'))
}

pub fn quoted_string<Input>() -> impl Parser<Input, Output = String>
    where Input : Stream<Token = char>,
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    let escaped_char = || token('\\').with(any());

    between(token('"'), token('"'), many(
        escaped_char()
        .or(any())
    ))
}