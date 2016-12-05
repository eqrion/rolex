use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::iter::Iterator;
use std::char;

use regex_syntax::*;

pub type NdfaStateId = usize;
pub type NdfaStateIdSet = BTreeSet<NdfaStateId>;

#[derive(Clone)]
pub struct NdfaState
{
  pub next : HashMap<char, NdfaStateId>,
  pub e : Vec<NdfaStateId>,
  pub answer : Option<usize>
}

impl NdfaState
{
  fn new_empty() -> NdfaState
  {
    NdfaState
    {
      next: HashMap::new(),
      e: Vec::new(),
      answer: None,
    }
  }
  fn new_directed(letter : char, index : NdfaStateId) -> NdfaState
  {
    let mut res = NdfaState::new_empty();
    res.next.insert(letter, index);
    return res;
  }
  fn new_e(index : NdfaStateId) -> NdfaState
  {
    let mut res = NdfaState::new_empty();
    res.e.push(index);
    return res;
  }
}

/*
 * An Ndfa. This is the starting class for generating a lexer, because
 * an Ndfa can be very naturally constructing from a regex. Ndfa's are
 * not useful to execute, but can be converted to an equivalent Dfa, which
 * can be executed easily.
 */
pub struct Ndfa
{
  pub states : Vec<NdfaState>
}
impl Ndfa
{
  fn new() -> Ndfa
  {
    Ndfa
    {
      states: Vec::new(),
    }
  }

  pub fn e_closure(&self, index: NdfaStateId) -> NdfaStateIdSet
  {
    let mut next: VecDeque<NdfaStateId> = VecDeque::new();
    let mut visited: HashSet<NdfaStateId> = HashSet::new();

    let mut result = NdfaStateIdSet::new();

    next.push_back(index);
    while !next.is_empty()
    {
      let cur = next.pop_front().unwrap();
      let state = self.states.get(cur).unwrap();

      for e in state.e.iter()
      {
        if !visited.contains(e)
        {
          visited.insert(*e);
          next.push_back(*e);
        }
      }

      result.insert(cur);
    }

    return result;
  }

  pub fn from_regexes(r : Vec<&Expr>) -> Ndfa
  {
    let mut res = Ndfa::new();

    res.states.push(NdfaState::new_empty());

    for (i, exp) in r.iter().enumerate()
    {
      let start = res.states.len();

      res.append_ndfa(&Ndfa::from_regex_with_answer(*exp, i));

      res.states.get_mut(0).unwrap().e.push(start);
    }

    return res;
  }

  #[allow(dead_code)]
  pub fn from_regex(r : &Expr) -> Ndfa
  {
    let mut res = Ndfa::build_regex_states(r);

    let last = res.states.len() - 1;
    res.states.get_mut(last).unwrap().answer = Some(1);

    return res;
  }

  pub fn from_regex_with_answer(r : &Expr, answer : usize) -> Ndfa
  {
    let mut res = Ndfa::build_regex_states(r);

    let last = res.states.len() - 1;
    res.states.get_mut(last).unwrap().answer = Some(answer);

    return res;
  }

  /*
   * Builds the states of an Ndfa corresponding to the regular expression r.
   * Note, this doesn't mark any states as final. That needs to be done by
   * a driver function. I didn't want to have to deal with managing the
   * final state (recursive calls make this messier), so the final state is
   * always implictly the last state in the Vec. (simplifies this code too)
   */
  fn build_regex_states(r : &Expr) -> Ndfa
  {
    match *r
    {
      Expr::Empty =>
      {
        let mut res = Ndfa::new();

        res.states.push(NdfaState::new_e(1));
        res.states.push(NdfaState::new_empty());

        return res;
      }

      Expr::Literal{ref chars, ref casei} =>
      {
        if *casei
        {
          unimplemented!();
        }

        let mut res = Ndfa::new();

        for (i, x) in chars.iter().enumerate()
        {
          res.states.push(NdfaState::new_directed(*x, i + 1));
        }
        res.states.push(NdfaState::new_empty());

        return res;
      }

      Expr::Class(ref c) =>
      {
        let mut res = Ndfa::new();

        res.states.push(NdfaState::new_empty());
        res.states.push(NdfaState::new_empty());

        for class in c.iter()
        {
          let first = res.states.get_mut(0).unwrap();

          let class_start = class.start as u32;
          let class_end = class.end as u32;

          for x in (class_start)..(class_end+1)
          {
            first.next.insert(char::from_u32(x).unwrap(), 1);
          }
        }

        return res;
      }

      Expr::Group{ref e, ..} =>
      {
        return Ndfa::build_regex_states(e.as_ref());
      }

      Expr::Repeat{ref e, ref r, ..} =>
      {
        match *r
        {
          Repeater::ZeroOrOne =>
          {
            let mut res = Ndfa::new();

            res.append_ndfa(&Ndfa::build_regex_states(e.as_ref()));

            let last_index = res.states.len() - 1;
            res.states.get_mut(0).unwrap().e.push(last_index);

            return res;
          },
          Repeater::ZeroOrMore =>
          {
            let mut res = Ndfa::new();

            res.append_ndfa(&Ndfa::build_regex_states(e.as_ref()));

            let last_index = res.states.len() - 1;
            res.states.get_mut(0).unwrap().e.push(last_index);
            res.states.get_mut(last_index).unwrap().e.push(0);

            return res;
          },
          Repeater::OneOrMore =>
          {
            return Ndfa::build_regex_states(
              &Expr::Concat(
                vec![
                  e.as_ref().clone(),
                  Expr::Repeat{
                    e: e.clone(),
                    r: Repeater::ZeroOrMore,
                    greedy: true,
                  }
                ]
                )
              );
          },
          Repeater::Range { min, max } =>
          {
            if let Some(max) = max
            {
              let mut alternates : Vec<Expr> = Vec::new();

              for i in min..(max+1)
              {
                alternates.push(
                  Expr::Concat((0..i).map(|_| e.as_ref().clone()).collect())
                  );
              }

              return Ndfa::build_regex_states(
                &Expr::Alternate(alternates)
                )
            }
            else
            {
              let mut concat : Vec<Expr> = (0..min).map(|_| e.as_ref().clone()).collect();

              concat.push(
                    Expr::Repeat{
                      e: e.clone(),
                      r: Repeater::ZeroOrMore,
                      greedy: true,
                    });

              return Ndfa::build_regex_states(
                &Expr::Concat(concat)
                );
            }
          }
        }
      }

      Expr::Concat(ref exprs) =>
      {
        assert!(exprs.len() > 0);

        let mut res = Ndfa::new();

        for e in exprs.iter()
        {
          res.append_ndfa(&Ndfa::build_regex_states(e));

          let last = res.states.len() - 1;
          res.states.get_mut(last).unwrap().e.push(last + 1);
        }

        res.states.push(NdfaState::new_empty());

        return res;
      }

      Expr::Alternate(ref exprs) =>
      {
        assert!(exprs.len() > 0);

        let mut res = Ndfa::new();
        let mut to_fixup = Vec::new();

        res.states.push(NdfaState::new_empty());

        for e in exprs.iter()
        {
          let start = res.states.len();

          res.append_ndfa(&Ndfa::build_regex_states(e));

          let end = res.states.len() - 1;

          res.states.get_mut(0).unwrap().e.push(start);
          to_fixup.push(end);
        }

        res.states.push(NdfaState::new_empty());

        let last = res.states.len() - 1;
        for id in to_fixup
        {
          res.states.get_mut(id).unwrap().e.push(last);
        }

        return res;
      }

      _ => unimplemented!()
    }
  }

  fn append_ndfa(&mut self, x: &Ndfa)
  {
    let base = self.states.len();

    for state in x.states.iter()
    {
      let mut state = state.clone();

      for (_, id) in state.next.iter_mut()
      {
        *id = *id + base;
      }
      for id in state.e.iter_mut()
      {
        *id = *id + base;
      }

      self.states.push(state);
    }
  }
}
