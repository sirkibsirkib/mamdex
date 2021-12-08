use core::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chunked_index_set::{ChunkRead, IndexSet};

type Var = u32;
type Val = u32;

#[derive(Debug, Clone)]
struct PartialState {
    assignments: HashMap<Var, Val>,
}

#[derive(Debug, Clone)]
struct Duty {
    name: &'static str,
    partial_state: PartialState,
}

#[derive(Debug, Clone)]
struct Action {
    name: &'static str,
    src_pstate: PartialState,
    dst_pstate: PartialState,
}

#[derive(Debug, Clone)]
struct Rule {
    name: &'static str,
    if_all: IndexSet<2>,
    then_all: IndexSet<2>,
    then_none: IndexSet<2>,
}
struct Specification {
    duties: Vec<Duty>,
    drules: Vec<Rule>,
    actions: Vec<Action>,
    arules: Vec<Rule>,
}

impl PartialState {
    // Can be understood as "self" matches pattern of "other"
    fn update(&mut self, other: &Self) {
        for (&var, &val) in other.assignments.iter() {
            self.assignments.insert(var, val);
        }
    }
}

enum PathNode {
    Start { start_state: PartialState },
    Next { prev: Arc<PathNode>, acts_indexes: IndexSet<2> },
}

/*
an pair of actions is mutually inconsistent IFF either:
- preconditions disagree
- preconditions discgree

*/

struct NextStateStepIter<'a> {
    prev: &'a Arc<PathNode>,
    spec: &'a Specification,
    next_subset_to_consider: IndexSet<2>,
}

///////////////////

impl PartialState {
    fn inconsistency_wrt(&self, other: &Self) -> Option<Var> {
        if self.assignments.len() > other.assignments.len() {
            other.inconsistency_wrt(self)
        } else {
            for (var, my_val) in self.assignments.iter() {
                match other.assignments.get(var) {
                    Some(other_val) if other_val != my_val => return Some(*var),

                    _ => {}
                }
            }
            None
        }
    }
}

impl PathNode {
    fn assignment(&self, spec: &Specification, var: Var) -> Option<Val> {
        match self {
            PathNode::Start { start_state } => start_state.assignments.get(&var).copied(),
            PathNode::Next { prev, acts_indexes } => {
                for act_index in acts_indexes.iter() {
                    if let Some(&val) = spec.actions[act_index].dst_pstate.assignments.get(&var) {
                        return Some(val);
                    }
                }
                prev.assignment(spec, var)
            }
        }
    }
    fn state_assigns_superset(
        &self,
        spec: &Specification,
        other: &PartialState,
    ) -> Result<(), Var> {
        for (&var, &other_val) in other.assignments.iter() {
            match self.assignment(spec, var) {
                None => return Err(var),
                Some(my_val) if my_val != other_val => return Err(var),
                Some(_) => {}
            }
        }
        Ok(())
    }
}

impl<'a> NextStateStepIter<'a> {
    fn new(prev: &'a Arc<PathNode>, spec: &'a Specification) -> Self {
        Self { prev, spec, next_subset_to_consider: (0..spec.actions.len()).collect() }
    }
}
impl Iterator for NextStateStepIter<'_> {
    type Item = PathNode;
    fn next(&mut self) -> Option<PathNode> {
        loop {
            if self.next_subset_to_consider.is_empty() {
                return None;
            }
            // try return this
            self.next_subset_to_consider.try_decrease_in_powerset_order();
        }
    }
}

impl Specification {
    fn paths_to_duty(&self, start_state: PartialState, duty_index: usize) -> Vec<PathNode> {
        let duty = &self.duties[duty_index];
        let mut incomplete = vec![PathNode::Start { start_state }];
        let mut complete = vec![];
        while let Some(path) = incomplete.pop() {
            if path.state_assigns_superset(self, &duty.partial_state).is_ok() {
                complete.push(path)
            } else {
                let prev = Arc::new(path);
                let next_iter = NextStateStepIter::new(&prev, self);
                incomplete.extend(next_iter);
            }
        }
        complete
    }
    /// does NOT deduplicate anything. Computes union of actions, duties, etc.
    fn subsume(&mut self, other: &Self) {
        self.duties.extend(other.duties.iter().cloned());
        self.actions.extend(other.actions.iter().cloned());
        self.arules.extend(other.arules.iter().cloned());
        self.drules.extend(other.drules.iter().cloned());
    }
}

fn main() {}
