use regex_syntax::Expr;

use ::ndfa::*;
use ::dfa::*;
use ::bdfa::*;

pub fn output_lex(tokens: Vec<(String, Expr)>, prefix: String) -> String
{
  let names: Vec<&str> = tokens.iter().map(|x| &x.0 as &str).collect();
  let regexes: Vec<&Expr> = tokens.iter().map(|x| &x.1).collect();

  let ndfa = Ndfa::from_regexes(regexes);
  let dfa = Dfa::from_ndfa(&ndfa);
  let bdfa = Bdfa::from_dfa(&dfa);

  let source_text = String::from(include_str!("templates/lex.rs"));

  return source_text
    .replace("$tokens$", &build_tokens(&names))
    .replace("$types$", &build_types(&names))
    .replace("$state-table-length$", &bdfa.states.len().to_string())
    .replace("$state-table$", &build_states(&bdfa))
    .replace("$answer-table$", &build_answers(&bdfa));
}

fn build_tokens(names: &Vec<&str>) -> String
{
  let mut res = String::new();

  for name in names.iter()
  {
    res.push_str(*name);
    res.push_str(",\n");
  }

  return res;
}
fn build_types(names: &Vec<&str>) -> String
{
  let mut res = String::new();

  for name in names.iter()
  {
    res.push_str(*name);
    res.push_str(",\n");
  }

  return res;
}
fn build_states(bdfa: &Bdfa) -> String
{
  let mut res = String::new();

  for state in bdfa.states.iter()
  {
    let mut line = String::new();
    line.push_str("[");

    for x in state.next.iter()
    {
      line.push_str(&x.to_string());
      line.push_str(", ");
    }

    line.push_str("],\n");
    res.push_str(&line);
  }

  return res;
}
fn build_answers(bdfa: &Bdfa) -> String
{
  let mut res = String::new();

  for state in bdfa.states.iter()
  {
    if let Some(answer) = state.answer
    {
      res.push_str(&answer.to_string());
    }
    else
    {
      res.push_str("-1");
    }
    res.push_str(", ")
  }

  return res;
}
