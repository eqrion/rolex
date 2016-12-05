use regex_syntax::Expr;

use ::ndfa::*;
use ::dfa::*;

pub fn output_lex(tokens: Vec<(String, Expr)>, prefix: String) -> String
{
  let names: Vec<&str> = tokens.iter().map(|x| &x.0 as &str).collect();
  let regexes: Vec<&Expr> = tokens.iter().map(|x| &x.1).collect();

  let ndfa = Ndfa::from_regexes(regexes);
  let dfa = Dfa::from_ndfa(&ndfa);

  let source_text = String::from(include_str!("templates/lex.js"));

  return source_text
      .replace("$prefix$", &prefix)
      .replace("$js-state-table$", &build_js_state_table(&dfa, &names));
}

fn escape_character(letter: char) -> String
{
  match letter
  {
    '\t' => String::from("'\\t'"),
    '\r' => String::from("'\\r'"),
    '\n' => String::from("'\\n'"),
    _ => format!("'{}'", letter),
  }
}

fn build_js_state_table(dfa: &Dfa, names: &Vec<&str>) -> String
{
  let mut res = String::new();

  for state in dfa.states.iter()
  {
    let mut line = String::new();

    line.push_str("{ ");

    for (letter, transition) in state.next.iter()
    {
      line.push_str(&escape_character(*letter));
      line.push_str(": ");
      line.push_str(&transition.to_string());
      line.push_str(", ");
    }

    if let Some(answer) = state.answer
    {
      line.push_str("answer: '");
      line.push_str(names.get(answer).unwrap());
      line.push_str("', ");
    }

    line.push_str(" },\n");

    res.push_str(&line);
  }

  return res;
}
