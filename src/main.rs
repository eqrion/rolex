#![feature(unicode)]

use std::io::{self, BufReader, BufRead, Write};
use std::fs::File;
use std::env;

extern crate getopts;
extern crate regex_syntax;
use getopts::Options;
use regex_syntax::*;

mod ndfa;
mod dfa;
mod bdfa;
mod lex;

fn valid_identifier(text: &str) -> bool
{
  let mut read_first = false;

  for c in text.chars()
  {
    if !read_first && !c.is_alphabetic()
    {
      return false;
    }
    else if !c.is_alphabetic() && !c.is_numeric()
    {
      return false;
    }

    read_first = true;
  }
  return true;
}

fn execute(input: Option<String>, output: Option<String>, prefix: Option<String>, target: lex::Target) -> i32
{
  let mut tokens = Vec::new();
  let mut number = 1;

  let stdin = io::stdin();
  let stdout = io::stdout();

  // Do some awfulness to get the file lines

  let lines = match input
  {
    Some(file) =>
    {
      match File::open(&file)
      {
        Ok(f) => BufReader::new(f),
        Err(_) =>
        {
          println!("error opening file {}.", file);
          return 1;
        }
      }.lines().filter(|x| x.is_ok()).map(|x| x.unwrap()).collect::<Vec<String>>()
    }
    None =>
    {
      stdin.lock().lines().filter(|x| x.is_ok()).map(|x| x.unwrap()).collect::<Vec<String>>()
    }
  };

  // Parse the rolex file

  for line in lines
  {
    let split_pos = match line.find(":")
    {
      Some(x) => x,
      None =>
      {
        println!("error on line {}: invalid token declaration.", number);
        return 1;
      }
    };

    let (identifier, pattern) = line.split_at(split_pos);

    // Handle the identifier

    let identifier = identifier.trim();

    if !valid_identifier(identifier)
    {
      println!("error on line {}: invalid token declaration. bad identifier.", number);
      return 1;
    }

    // Handle the regex

    let pattern = pattern.split_at(1).1.trim();

    let regex = match Expr::parse(pattern)
    {
      Ok(x) => x,
      Err(e) =>
      {
        println!("error on line {}: invalid token declaration. bad pattern.", e.to_string());
        return 1;
      }
    };

    tokens.push((String::from(identifier), regex));
    number += 1;
  }

  let prefix = match prefix
  {
    Some(p) => p,
    None => String::from("")
  };

  let lex_source = lex::output_lex(tokens, prefix, target);

  // Print the output

  match output
  {
    Some(file) =>
    {
      match File::create(&file)
      {
        Ok(mut f) =>
        {
          f.write_all(&lex_source.as_bytes()).unwrap();
        },
        Err(_) =>
        {
          println!("error opening file {}.", file);
          return 1;
        }
      }
    },
    None =>
    {
      stdout.lock().write_all(&lex_source.as_bytes()).unwrap();
    }
  }

  return 0;
}

fn main()
{
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();

  opts.optopt("i", "", "specify an input filename, defaults to stdin.", "NAME");
  opts.optopt("o", "", "specify an output filename, defaults to stdout.", "NAME");
  opts.optopt("t", "target", "specify target type. defaults to c.", "TYPE");
  opts.optopt("p", "prefix", "specify the generated parser prefix.", "PREFIX");
  opts.optflag("h", "help", "print this help menu.");

  let matches = match opts.parse(&args[1..])
  {
    Ok(m) => m,
    Err(f) =>
    {
      println!("error: invalid command line arguments. {}", f.to_string());
      std::process::exit(1);
    }
  };

  // Go through the flags

  if matches.opt_present("h")
  {
    println!("{}", opts.usage(&format!("Usage: {} INPUT [options]", program)));
    return;
  }

  let input = match matches.opt_str("i")
  {
    Some(x) => Some(x),
    None =>
    {
      if matches.free.len() == 1
      {
        Some(matches.free[0].clone())
      }
      else
      {
        None
      }
    }
  };

  let output = matches.opt_str("o");

  let target = match matches.opt_str("t")
  {
    Some(text) =>
    {
      match lex::Target::parse(&text)
      {
        Some(t) => t,
        None =>
        {
          println!("error: invalid target {}", text);
          std::process::exit(1);
        }
      }
    },
    None =>
    {
      if let Some(ref output) = output
      {
        if let Some(target) = lex::Target::guess(output)
        {
          target
        }
        else
        {
          lex::Target::C
        }
      }
      else
      {
        lex::Target::C
      }
    }
  };

  let prefix = matches.opt_str("p");

  std::process::exit(execute(input, output, prefix, target));
}

#[test]
fn regex_test()
{
  let regex = Expr::parse(r"-?[0-9]+(\.[0-9]*)?").unwrap();
  let ndfa = ndfa::Ndfa::from_regex(&regex);
  let dfa = dfa::Dfa::from_ndfa(&ndfa);
  let bdfa = bdfa::Bdfa::from_dfa(&dfa);

  assert!(!bdfa.accepts("").is_some());
  assert!(!bdfa.accepts("hello").is_some());
  assert!(!bdfa.accepts("0.f4").is_some());

  assert!(bdfa.accepts("0").is_some());
  assert!(bdfa.accepts("-0").is_some());
  assert!(bdfa.accepts("0.").is_some());
  assert!(bdfa.accepts("-0.").is_some());
  assert!(bdfa.accepts("0.1").is_some());
  assert!(bdfa.accepts("-0.1").is_some());
}

#[test]
fn emoji_test()
{
  let regex = Expr::parse(r"(😎|🙁)+").unwrap();
  let ndfa = ndfa::Ndfa::from_regex(&regex);
  let dfa = dfa::Dfa::from_ndfa(&ndfa);
  let bdfa = bdfa::Bdfa::from_dfa(&dfa);

  assert!(!bdfa.accepts("").is_some());
  assert!(!bdfa.accepts("😤").is_some());
  assert!(!bdfa.accepts("hello").is_some());

  assert!(bdfa.accepts("😎").is_some());
  assert!(bdfa.accepts("🙁").is_some());
  assert!(bdfa.accepts("😎🙁").is_some());
  assert!(bdfa.accepts("🙁😎").is_some());
  assert!(bdfa.accepts("🙁😎😎").is_some());
  assert!(bdfa.accepts("😎🙁😎").is_some());
  assert!(bdfa.accepts("😎😎🙁").is_some());
}

// #[test]
// fn lex_test()
// {
//    let regexes = vec![
//      Expr::parse(r"-?[0-9]+(\.[0-9]*)?").unwrap(),    // number
//      Expr::parse(r"[a-zA-Z][a-zA-Z0-9]*").unwrap(),   // identifier
//      Expr::parse(r"[ \t]+").unwrap(),                 // whitespace
//      Expr::parse(r"\n|(\r\n)").unwrap(),              // newline
//    ];

//    let ndfa = Ndfa::from_regexes(&regexes);
//    let dfa = Dfa::from_ndfa(&ndfa);
//    let bdfa = Bdfa::from_dfa(&dfa);

//    let text = "hello 7.7 world -4.5\nthis 0 is a test";
// }
