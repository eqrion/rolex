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

  let source_text = String::from(include_str!("templates/lex.c"));

  return source_text
    .replace("$prefix$", &prefix)
    .replace("$c-token-enum$", &build_c_token_enum(&prefix, &names))
    .replace("$c-state-table$", &build_c_state_table(&bdfa))
    .replace("$c-answer-table$", &build_c_answer_table(&bdfa));
}

// C Templates

fn build_c_state_table(bdfa: &Bdfa) -> String
{
  let mut res = String::new();

  for state in bdfa.states.iter()
  {
    let mut line = String::new();
    line.push_str("{ ");

    for x in state.next.iter()
    {
      line.push_str(&x.to_string());
      line.push_str(", ");
    }

    line.push_str(" },\n");
    res.push_str(&line);
  }

  return res;
}
fn build_c_answer_table(bdfa: &Bdfa) -> String
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
fn build_c_token_enum(prefix: &str, names: &Vec<&str>) -> String
{
  let mut res = String::new();

  for name in names.iter()
  {
    res.push_str(prefix);
    res.push_str("token_");
    res.push_str(*name);
    res.push_str(",\n");
  }

  return res;
}
