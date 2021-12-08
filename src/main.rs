use chunked_index_set::{ChunkRead, IndexSet};
use maplit::hashmap as hm;
use std::{collections::HashMap, sync::Arc};

macro_rules! zrintln {
    ($($arg:tt)*) => ({
        println!( $( $arg ) * );
    })
}

type Var = u32;
type Val = u32;

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
struct Specification {
    duties: Vec<Duty>,
    drules: Vec<Rule>,
    actions: Vec<Action>,
    arules: Vec<Rule>,
}

#[derive(Debug, Clone)]
enum PathNode {
    Start { start_state: PartialState },
    Next { prev: Arc<PathNode>, acts_indexes: IndexSet<2> },
}

/*
an pair of actions is mutually inconsistent IFF either:
- preconditions disagree
- preconditions discgree
*/

#[derive(Debug)]
struct NextStateStepIter<'a> {
    prev: &'a Arc<PathNode>,
    spec: &'a Specification,
    next_subset_to_consider: IndexSet<2>,
}

///////////////////

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
    fn try_create_next_step(
        me: &Arc<Self>,
        acts_indexes: &IndexSet<2>,
        spec: &Specification,
    ) -> Option<Self> {
        // 1: check that all action rules are OK
        for arule in spec.arules.iter() {
            if arule.if_all.is_superset_of(acts_indexes)
                && (!arule.then_all.is_subset_of(acts_indexes)
                    || !arule.then_none.is_disjoint_with(acts_indexes))
            {
                zrintln!("rule {:?} mismatch", arule);
                return None;
            }
        }
        // 2: check that all action post conditions are consistent
        if !spec.postconditions_consistent(acts_indexes) {
            zrintln!("postconditions_inconsistent");
            return None;
        }
        // 3: check that all action preconditions are OK
        for act_index in acts_indexes.iter() {
            let action = &spec.actions[act_index];
            if let Err(var) = me.state_assigns_superset(spec, &action.src_pstate) {
                zrintln!("preconds bad {:?}", var);
                return None;
            }
        }
        Some(Self::Next { prev: me.clone(), acts_indexes: acts_indexes.clone() })
    }
}

impl<'a> NextStateStepIter<'a> {
    fn new(prev: &'a Arc<PathNode>, spec: &'a Specification) -> Self {
        let me = Self { prev, spec, next_subset_to_consider: (0..spec.actions.len()).collect() };
        zrintln!("next_subset_to_consider: {:?}", me.next_subset_to_consider);
        me
    }
}
impl Iterator for NextStateStepIter<'_> {
    type Item = PathNode;
    fn next(&mut self) -> Option<PathNode> {
        loop {
            zrintln!("consider {:#?}", self);
            if self.next_subset_to_consider.is_empty() {
                return None;
            }
            let next =
                PathNode::try_create_next_step(self.prev, &self.next_subset_to_consider, self.spec);
            self.next_subset_to_consider.try_decrease_in_powerset_order();
            if next.is_some() {
                return next;
            }
        }
    }
}

impl Specification {
    fn postconditions_consistent(&self, action_indexes: &IndexSet<2>) -> bool {
        let mut delta = PartialState::default();
        for action_index in action_indexes.iter() {
            let action = &self.actions[action_index];
            for (&var, &action_val) in action.dst_pstate.assignments.iter() {
                match delta.assignments.insert(var, action_val) {
                    Some(delta_val) if action_val != delta_val => return false,
                    _ => {}
                }
            }
        }
        true
    }
    fn paths_to_duty(&self, start_state: PartialState, duty_index: usize) -> Vec<PathNode> {
        let duty = &self.duties[duty_index];
        let mut incomplete = vec![PathNode::Start { start_state }];
        let mut complete = vec![];
        while let Some(path) = incomplete.pop() {
            zrintln!("next path {:?}", &path);
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

fn main() {
    let s = Specification {
        actions: vec![
            // yass
            Action {
                name: "Var(0) := 3",
                src_pstate: PartialState { assignments: hm! {} },
                dst_pstate: PartialState { assignments: hm! { 0 => 3 } },
            },
        ],
        arules: vec![],
        drules: vec![],
        duties: vec![Duty {
            name: "Var(0) == 3",
            partial_state: PartialState {
                assignments: hm! {
                    0 => 3,
                },
            },
        }],
    };
    let start_state = PartialState { assignments: hm! { 0 => 0, 1 => 1} };
    let duty_index = 0;
    let r = s.paths_to_duty(start_state, duty_index);
    zrintln!("r {:#?}", r);
}
