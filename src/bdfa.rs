use std::collections::HashMap;
use super::dfa::*;

pub type BdfaStateId = isize;
pub struct BdfaState
{
  pub next : [BdfaStateId; 256],
  pub answer : Option<usize>
}
impl BdfaState
{
  fn new() -> BdfaState
  {
    BdfaState
    {
      next: [-1; 256],
      answer: None,
    }
  }
}

/*
 * A Byte-Based DFA. This module generates a DFA that can accept UTF-8 strings,
 * without using a hashtable or such. It expands the state table of another DFA
 * to decode UTF-8.
 */
pub struct Bdfa
{
  pub states : Vec<BdfaState>,
}
impl Bdfa
{
  fn new() -> Bdfa
  {
    Bdfa
    {
      states: Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub fn accepts(&self, text: &str) -> Option<usize>
  {
    let mut cur: BdfaStateId = 0;

    for byte in text.as_bytes()
    {
      let state = self.states.get(cur as usize).unwrap();

      match state.next[*byte as usize]
      {
        -1 => return None,
        x => cur = x,
      }
    }

    return self.states.get(cur as usize).unwrap().answer;
  }

  pub fn from_dfa(dfa : &Dfa) -> Bdfa
  {
    let (temp_states, translation) = TempState::convert(&dfa.states);

    let mut res = Bdfa::new();

    for state in temp_states.iter()
    {
      res.states.push(state.translate(&translation));
    }

    return res;
  }
}

/*
 * The motivation for the following helper class is weird. I made the design decision
 * to represent DFAs in arrays with transitions as indices into that array. This makes
 * expansion and other construction techniques hard because you can invalidate a state's
 * transition if you insert into the middle.
 *
 * The purpose of this module is to have an intermediate format that I can expand states in,
 * while marking transitions to be fix up-ed at the end, and transitions that are now valid.
 *
 * This would be easier if we were only dealing with an in memory linked structure.
 */

#[derive(Copy, Clone)]
enum TempTransition
{
  None,
  Translated(DfaStateId),
  Untranslated(DfaStateId),
}
struct TempState
{
  next : [TempTransition; 256],
  answer : Option<usize>
}
impl TempState
{
  fn new() -> TempState
  {
    TempState
    {
      next: [TempTransition::None; 256],
      answer: None,
    }
  }

  fn translate(&self, translation: &HashMap<DfaStateId, usize>) -> BdfaState
  {
    let mut result = BdfaState::new();

    for (i, transition) in self.next.iter().enumerate()
    {
      result.next[i] = match *transition {
          TempTransition::None => -1,
          TempTransition::Translated(x) => x as BdfaStateId,
          TempTransition::Untranslated(x) => *translation.get(&x).unwrap() as BdfaStateId,
        };
    }
    result.answer = self.answer;

    return result;
  }

  fn convert(states: &Vec<DfaState>) -> (Vec<TempState>, HashMap<DfaStateId, usize>)
  {
    let mut output = Vec::new();
    let mut translation = HashMap::new();

    // Convert every state, keeping a translation marking where the original maps to

    for (i, s) in states.iter().enumerate()
    {
      let root_index = output.len();
      let decoded_transitions = s.next.iter().map(|x| (EncodeUtf8::new(*x.0), *x.1)).collect();

      TempState::emit(
        decoded_transitions,
        &mut output);

      translation.insert(i, root_index);
      output.get_mut(root_index).unwrap().answer = s.answer;
    }

    return (output, translation)
  }

  fn emit(mut transition_list: Vec<(EncodeUtf8, DfaStateId)>, output: &mut Vec<TempState>)
  {
    // A list of unfinished transitions by root prefix

    let mut deferred = HashMap::<u8, Vec<(EncodeUtf8, DfaStateId)>>::new();

    // Build the state from this step from transitions with only one byte left
    // Mark the transitions that we need to handle with a new state

    let mut cur = TempState::new();

    for (mut bytes, transition) in transition_list.drain(0..)
    {
      if bytes.as_slice().len() == 1
      {
        // We are at the end of the byte string, link back to the original dfa

        let byte = bytes.next().unwrap();
        cur.next[byte as usize] = TempTransition::Untranslated(transition);
      }
      else if bytes.as_slice().len() > 1
      {
        // We have to make another state for this, mark it for later

        let byte = bytes.next().unwrap();

        if deferred.contains_key(&byte)
        {
          deferred.get_mut(&byte).unwrap().push((bytes, transition));
        }
        else
        {
          deferred.insert(byte, vec![(bytes, transition)]);
        }
      }
      else
      {
        unreachable!();
      }
    }

    // Add the state

    let root_index = output.len();
    output.push(cur);

    // Create and link to all the new states from common prefixes

    for (byte, transitions) in deferred.drain()
    {
      let index = output.len();

      TempState::emit(transitions, output);
      output.get_mut(root_index).unwrap().next[byte as usize] = TempTransition::Translated(index);
    }
  }
}

struct EncodeUtf8
{
  buffer: [u8; 4],
  len: usize,
  cur: usize,
}
impl EncodeUtf8
{
  fn new(character: char) -> EncodeUtf8
  {
    let mut res = EncodeUtf8 {
      buffer: [0; 4],
      len: 0,
      cur: 0,
    };

    res.len = character.encode_utf8(&mut res.buffer).len();

    res
  }

  fn next(&mut self) -> Option<u8>
  {
    if self.cur >= self.len {
      return None;
    }

    let res = self.buffer[self.cur];
    self.cur += 1;
    return Some(res);
  }

  fn as_slice(&self) -> &[u8]
  {
    &self.buffer[self.cur..self.len]
  }
}
