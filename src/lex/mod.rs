use regex_syntax::Expr;
use std::path::Path;

mod js;
mod rs;
mod c;

pub enum Target
{
    Rust,
    JavaScript,
    C,
}
impl Target
{
  pub fn parse(text: &str) -> Option<Target>
  {
    if text == "rs" || text == "rust" {
      Some(Target::Rust)
    } else if text == "js" || text == "javascript" {
      Some(Target::JavaScript)
    } else if text == "c" {
      Some(Target::C)
    } else {
      None
    }
  }

  pub fn guess(filename: &str) -> Option<Target>
  {
    let path = Path::new(filename);
    let extension = path.extension();

    match extension {
      Some(x) if x == "rs" => Some(Target::Rust),
      Some(x) if x == "js" => Some(Target::JavaScript),
      Some(x) if x == "c" => Some(Target::C),
      Some(_) => None,
      None => None,
    }
  }
}

pub fn output_lex(tokens: Vec<(String, Expr)>, mut prefix: String, target: Target) -> String
{
  match target
  {
    Target::Rust => rs::output_lex(tokens, prefix),
    Target::JavaScript => js::output_lex(tokens, prefix),
    Target::C =>
    {
      if prefix != ""
      {
        prefix.push('_');
      }

      c::output_lex(tokens, prefix)
    }
  }
}
