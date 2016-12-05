use std::str::from_utf8;

const TRANSITIONS: [[i8; 256]; $state-table-length$] = [
$state-table$];
const ANSWERS: [isize; $state-table-length$] = [ $answer-table$ ];

pub enum Token
{
$tokens$}
impl Token
{
  fn parse(token_type: isize, text: &str) -> Token
  {
    unimplemented!();
  }
}

pub struct Lexer<'a>
{
  bytes: &'a [u8],
  consumed: usize,
}
impl<'a> Lexer<'a>
{
  pub fn new(text: &'a str) -> Lexer<'a>
  {
    Lexer {
      bytes: text.as_bytes(),
      consumed: 0,
    }
  }

  pub fn tokenize(&mut self) -> Result<Vec<Token>, usize>
  {
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(parse) = self.next()
    {
      match parse
      {
        Ok(x) => tokens.push(x),
        Err(x) => return Err(x),
      }
    }

    return Ok(tokens);
  }
}

impl<'a> Iterator for Lexer<'a>
{
  type Item = Result<Token, usize>;

  fn next(&mut self) -> Option<Result<Token, usize>>
  {
    if self.bytes.len() == 0 {
      return None;
    }

    let mut state: usize = 0;
    let mut marker: usize = 0;
    let mut best_match: Option<(usize, isize)> = None;

    if ANSWERS[state] != -1
    {
      best_match = Some((marker, ANSWERS[state]));
    }

    while marker < self.bytes.len()
    {
      let next_state = TRANSITIONS[state][self.bytes[marker] as usize];

      if next_state == -1 {
        break;
      }

      state = next_state as usize;
      marker += 1;

      if ANSWERS[state] != -1
      {
        best_match = Some((marker, ANSWERS[state]));
      }
    }

    if let Some((marker, token_type)) = best_match
    {
      let text = from_utf8(&self.bytes[..marker]).unwrap();
      let token = Token::parse(token_type, text);

      self.consumed += marker;
      self.bytes = &self.bytes[marker..];

      return Some(Ok(token));
    }
    return Some(Err(self.consumed + marker - 1));
  }
}
