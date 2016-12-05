use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::collections::BinaryHeap;
use std::iter::Iterator;

use ::ndfa::*;

pub type DfaStateId = usize;
pub struct DfaState
{
  pub next : BTreeMap<char, DfaStateId>,
  pub answer : Option<usize>
}
impl DfaState
{
  fn new() -> DfaState
  {
    DfaState
    {
      next: BTreeMap::new(),
      answer: None,
    }
  }
}

/*
 * A Dfa. The Dfa is a list of states, each with a table of transitions
 * that takes a Unicode codepoint and gives the next state's index in the list.
 * Each state also has an optional answer field to indicate whether this is
 * an accepting state or not.
 */
pub struct Dfa
{
  pub states : Vec<DfaState>,
}
impl Dfa
{
  fn new() -> Dfa
  {
    Dfa
    {
      states: Vec::new(),
    }
  }

  #[allow(dead_code)]
  pub fn accepts(&self, text: &str) -> Option<usize>
  {
    let mut cur: DfaStateId = 0;

    for letter in text.chars()
    {
      let state = self.states.get(cur).unwrap();

      match state.next.get(&letter)
      {
        Some(next) => cur = *next,
        None => return None,
      }
    }

    return self.states.get(cur).unwrap().answer;
  }

  pub fn from_ndfa(ndfa : &Ndfa) -> Dfa
  {
    let mut to_visit: VecDeque<NdfaStateIdSet> = VecDeque::new();
    let mut visited: HashMap<NdfaStateIdSet, usize> = HashMap::new();

    let mut result: Dfa = Dfa::new();

    let start = ndfa.e_closure(0);
    visited.insert(start.clone(), 0);
    to_visit.push_back(start);

    while !to_visit.is_empty()
    {
      let current_sset = to_visit.pop_front().unwrap();

      // Grab every transition arrow into a big list

      let mut all_transitions: Vec<(char, NdfaStateId)> = Vec::new();

      for id in current_sset.iter()
      {
        let state = ndfa.states.get(*id).unwrap();

        for (letter, id) in state.next.iter()
        {
          all_transitions.push((*letter, *id));
        }
      }

      // Group transition arrows by which token they are used for
      // This creates essentially what the next big state is for
      // if this token was read.

      let mut grouped_transitions: HashMap<char, NdfaStateIdSet> = HashMap::new();

      for (letter, index) in all_transitions
      {
        if !grouped_transitions.contains_key(&letter)
        {
          grouped_transitions.insert(letter, NdfaStateIdSet::new());
        }

        for id in ndfa.e_closure(index).iter()
        {
          grouped_transitions.get_mut(&letter).unwrap().insert(*id);
        }
      }

      // We can now construct the new state by 'recursing'

      let mut resulting_state = DfaState::new();

      // Build the transitions, enqueuing work that needs to be done

      for (letter, sset) in grouped_transitions
      {
        if visited.contains_key(&sset)
        {
          let transition = *visited.get(&sset).unwrap();

          resulting_state.next.insert(letter, transition);
        }
        else
        {
          let future_index = (result.states.len() + 1) + to_visit.len();

          resulting_state.next.insert(letter, future_index);

          visited.insert(sset.clone(), future_index);
          to_visit.push_back(sset.clone());
        }
      }

      // Detect whether this should be accept or not

      resulting_state.answer = current_sset.iter().filter_map(|&x| ndfa.states.get(x).unwrap().answer).collect::<BinaryHeap<_>>().pop();

      // Add it

      result.states.push(resulting_state);
    };

    return result;
  }
}
