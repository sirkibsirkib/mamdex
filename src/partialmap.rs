use chunked_index_set::ChunkRead;
use maplit::hashmap as hm;
use std::borrow::Cow;
use std::{collections::HashMap, sync::Arc};

type IndexSet = chunked_index_set::IndexSet<1>;

macro_rules! zrintln {
    ($($arg:tt)*) => ({
        println!( $( $arg ) * );
    })
}
macro_rules! zrint {
    ($($arg:tt)*) => ({
        print!( $( $arg ) * );
    })
}

type Var = u32;
type Val = u32;

/// Represents partial assignment of var to val, i.e., Var -> 0 | Val
#[derive(Debug, Clone, Default)]
pub struct PartialState {
    assignments: HashMap<Var, Val>,
}

/// Named predicate over states. Satisfied by all states whose assignments are a superset of these
#[derive(Debug, Clone)]
pub struct Duty {
    pub name: &'static str,
    pub partial_state: PartialState,
}

/// Named predicate over transitions, i.e., (PartialState,PartialState).
/// Precondition is a predicate over the source and postcondition is a predicate over dest.
#[derive(Debug, Clone)]
pub struct Action {
    pub name: &'static str,
    pub src_pstate: PartialState,
    pub dst_pstate: PartialState,
}

/// Named requirement constraining the permitted combinations of indices.
/// Satisfied = (if_all is subset) -> (then_all is subset && then_none is disjoint).
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: &'static str,
    pub if_all: IndexSet,
    pub then_all: IndexSet,
    pub then_none: IndexSet,
}

/// Collection of duties, duty rules, actions, action rules.
#[derive(Debug, Clone, Default)]
pub struct Specification {
    pub duties: Vec<Duty>,
    pub drules: Vec<Rule>,
    pub actions: Vec<Action>,
    pub arules: Vec<Rule>,
}

////////////////////
#[derive(Debug, Clone)]
enum PathNode {
    Start { start_state: PartialState },
    Next { prev: Arc<PathNode>, acts_indexes: IndexSet },
}

#[derive(Debug)]
struct NextStateStepIter<'a> {
    prev: &'a Arc<PathNode>,
    spec: &'a Specification,
    next_subset_to_consider: IndexSet,
}

#[derive(Debug)]
enum StepError {
    ViolatesActionRule { action_rule_index: usize },
    ViolatesActionPrecondition { action_index: usize, var: Var },
    ConflictingAssignments { var: Var },
    ViolatesDutyRule { duty_rule_index: usize },
}

trait ReadableState {
    fn assignment(&self, spec: &Specification, var: Var) -> Option<Val>;
    fn to_partial_state(&self, spec: &Specification) -> Cow<PartialState>;
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
    fn state_satisfies_duty(&self, spec: &Specification, duty: &Duty) -> Result<(), Var> {
        self.state_assigns_superset(spec, &duty.partial_state)
    }
    fn satisfied_duties(&self, spec: &Specification) -> IndexSet {
        (0..spec.duties.len())
            .filter(|&duty_index| self.state_satisfies_duty(spec, &spec.duties[duty_index]).is_ok())
            .collect()
    }
    fn violated_duty_rule(&self, spec: &Specification) -> Option<usize> {
        let satisfied_duties = self.satisfied_duties(spec);
        spec.drules
            .iter()
            .enumerate()
            .find(|(_, duty_rule)| !duty_rule.satisfied_by(&satisfied_duties))
            .map(|(duty_rule_index, _)| duty_rule_index)
    }
}

///////////////////

impl Rule {
    fn satisfied_by(&self, indexes: &IndexSet) -> bool {
        // trivial closures enable short-circuiting, while avoiding having to inline the bodies
        let if_all = || self.if_all.is_subset_of(indexes);
        let then_all = || self.then_all.is_subset_of(indexes);
        let else_none = || self.then_none.is_disjoint_with(indexes);
        !if_all() || (then_all() && else_none())
    }
}

impl ReadableState for PartialState {
    fn assignment(&self, _spec: &Specification, var: Var) -> Option<Val> {
        self.assignments.get(&var).copied()
    }
    fn to_partial_state(&self, _spec: &Specification) -> Cow<PartialState> {
        Cow::Borrowed(self)
    }
}
impl ReadableState for PathNode {
    fn assignment(&self, spec: &Specification, var: Var) -> Option<Val> {
        match self {
            PathNode::Start { start_state } => start_state.assignment(spec, var),
            PathNode::Next { prev, acts_indexes } => {
                for act_index in acts_indexes.iter() {
                    if let Some(val) = spec.actions[act_index].dst_pstate.assignment(spec, var) {
                        return Some(val);
                    }
                }
                prev.assignment(spec, var)
            }
        }
    }
    fn to_partial_state(&self, spec: &Specification) -> Cow<PartialState> {
        let mut c = PartialState::default();
        let mut n = self;
        loop {
            match n {
                Self::Start { start_state } => {
                    for (&var, &val) in start_state.assignments.iter() {
                        c.assignments.entry(var).or_insert(val);
                    }
                    return Cow::Owned(c);
                }
                Self::Next { prev, acts_indexes } => {
                    for (var, val) in spec.actions_assignments(&acts_indexes) {
                        c.assignments.entry(var).or_insert(val);
                    }
                    n = prev;
                }
            }
        }
    }
}

impl PathNode {
    fn zrinty_up(&self, spec: &Specification) -> PartialState {
        match self {
            Self::Start { start_state } => {
                zrint!("{:?}", start_state);
                start_state.clone()
            }
            Self::Next { prev, acts_indexes } => {
                let mut state = prev.zrinty_up(spec);
                for (var, val) in spec.actions_assignments(acts_indexes) {
                    state.assignments.insert(var, val);
                }
                zrint!("\n=={:?}==> {:?}", acts_indexes, state);
                state
            }
        }
    }
    fn try_create_next_step(
        me: &Arc<Self>,
        acts_indexes: &IndexSet,
        spec: &Specification,
    ) -> Result<Self, StepError> {
        // 1: check that all action rules are OK
        if let Some((action_rule_index, _)) = spec
            .arules
            .iter()
            .enumerate()
            .find(|(_, action_rule)| !action_rule.satisfied_by(acts_indexes))
        {
            return Err(StepError::ViolatesActionRule { action_rule_index });
        }

        // 2: check that all action preconditions are satisfied by our state
        for action_index in acts_indexes.iter() {
            let action = &spec.actions[action_index];
            if let Err(var) = me.state_assigns_superset(spec, &action.src_pstate) {
                return Err(StepError::ViolatesActionPrecondition { action_index, var });
            }
        }

        // 3: check that all action post conditions are mutually consistent
        if let Err(var) = spec.consistent_assignments(acts_indexes) {
            return Err(StepError::ConflictingAssignments { var });
        }

        // 4: check that destination state satisfies all duty rules
        let new = Self::Next { prev: me.clone(), acts_indexes: acts_indexes.clone() };
        if let Some(duty_rule_index) = new.violated_duty_rule(spec) {
            return Err(StepError::ViolatesDutyRule { duty_rule_index });
        }
        // ok! return the result of taking this step
        Ok(new)
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
            let result_next =
                PathNode::try_create_next_step(self.prev, &self.next_subset_to_consider, self.spec);
            zrintln!("result next {:?}", &result_next);
            let maybe_next = result_next.ok();
            self.next_subset_to_consider.try_decrease_in_powerset_order();
            if maybe_next.is_some() {
                return maybe_next;
            }
        }
    }
}

impl Specification {
    fn actions_assignments<'a>(
        &'a self,
        action_indexes: &'a IndexSet,
    ) -> impl Iterator<Item = (Var, Val)> + 'a {
        action_indexes.iter().flat_map(|action_index| {
            let action = &self.actions[action_index];
            action.dst_pstate.assignments.iter().map(|(&a, &b)| (a, b))
        })
    }
    fn consistent_assignments(&self, action_indexes: &IndexSet) -> Result<PartialState, Var> {
        let mut c = PartialState::default();
        for (var, action_val) in self.actions_assignments(action_indexes) {
            match c.assignments.insert(var, action_val) {
                Some(delta_val) if action_val != delta_val => return Err(var),
                _ => {}
            }
        }
        Ok(c)
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
    /// Returns the composition of this specification with another, without mutating either.
    /// Does NOT check for name collisions between contents; result is the union of respective contents.
    pub fn compose(&self, other: &Self) -> Self {
        fn f<T: Clone>(a: &Vec<T>, b: &Vec<T>) -> Vec<T> {
            a.iter().chain(b.iter()).cloned().collect()
        }
        Self {
            duties: f(&self.duties, &other.duties),
            drules: f(&self.drules, &other.drules),
            actions: f(&self.actions, &other.actions),
            arules: f(&self.arules, &other.arules),
        }
    }
}

#[test]
fn path_test() {
    let spec = Specification {
        duties: vec![
            Duty {
                name: "Var(0) == 3",
                partial_state: PartialState {
                    assignments: hm! {
                        0 => 3
                    },
                },
            },
            Duty {
                name: "Var(99) == 99",
                partial_state: PartialState {
                    assignments: hm! {
                        99 => 99
                    },
                },
            },
        ],
        actions: vec![
            Action {
                name: "Var(99) := 99",
                src_pstate: PartialState { assignments: hm! {} },
                dst_pstate: PartialState { assignments: hm! { 99 => 99 } },
            },
            Action {
                name: "Var(0) := 3",
                src_pstate: PartialState { assignments: hm! {} },
                dst_pstate: PartialState { assignments: hm! { 0 => 3 } },
            },
        ],
        drules: vec![
            // the only duty rule
            Rule {
                name: "Duty 1 always FALSE",
                if_all: Default::default(),
                then_all: Default::default(),
                then_none: std::iter::once(1).collect(),
            },
        ],
        arules: vec![],
    };
    let start_state = PartialState { assignments: hm! { 0 => 0, 1 => 1} };

    // run!
    let duty_index = 0;
    let r = spec.paths_to_duty(start_state, duty_index);

    zrintln!("===================");
    for (i, q) in r.iter().enumerate() {
        zrintln!("path {}", i);
        q.zrinty_up(&spec);
        zrintln!("\n");
    }
}
