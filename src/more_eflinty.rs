use chunked_index_set::ChunkRead;
use core::marker::PhantomData;
use std::collections::HashSet;
use std::sync::Arc;

/*
let's take the union approach; a fact is true iff its postulated OR there's 1+ ways to derive it
the essence of a state is the set of postulated facts
we don't need non-boolean predicates; they are just partial functions we can worry about later
duties are just facts marked as "necessary for satisfaction". let's worry about that later
*/

struct Atom {
	data: u32
}
struct Fact {
	Arc<[Atom]>
}
struct FactSet {
	HashSet<Fact>
}
struct State {

}

type IndexSet = chunked_index_set::IndexSet<1>;

#[derive(Debug)]
struct SpecStore<T> {
    data: Vec<T>,
}
impl<T> SpecStore<T> {
    fn get(&self, key: SpecKey<T>) -> Option<&T> {
        self.data.get(key.index)
    }
}
impl<T> Default for SpecStore<T> {
    fn default() -> Self {
        Self { data: Default::default() }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct SpecKey<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Atom {
    data: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Fact {
    atoms: Arc<[Atom]>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FactSet {
    facts: HashSet<Fact>,
}

enum StateToState {
    Identity,
    Constant(Arc<FactSet>),
    UnionOf([Arc<StateToState>; 2]),
    IntersectionOf([Arc<StateToState>; 2]),
    DifferenceWith([Arc<StateToState>; 2]),
}
impl StateToState {
    fn flatten(&self, state: Arc<FactSet>) -> Arc<FactSet> {
        match self {
            Self::Identity => state,
            Self::Constant(x) => x.clone(),
            Self::UnionOf([a, b]) => Arc::new(FactSet {
                facts: a
                    .flatten(state.clone())
                    .facts
                    .union(&b.flatten(state).facts)
                    .cloned()
                    .collect(),
            }),
            Self::IntersectionOf([a, b]) => Arc::new(FactSet {
                facts: a
                    .flatten(state.clone())
                    .facts
                    .intersection(&b.flatten(state).facts)
                    .cloned()
                    .collect(),
            }),
            Self::DifferenceWith([a, b]) => Arc::new(FactSet {
                facts: a
                    .flatten(state.clone())
                    .facts
                    .difference(&b.flatten(state).facts)
                    .cloned()
                    .collect(),
            }),
        }
    }
}

enum StateToBool {
    Const(bool),
    Empty,
    Not(Arc<StateToBool>),
}
impl StateToBool {
    fn eval(&self, state: Arc<FactSet>) -> bool {
        match self {
            Self::Empty => state.facts.is_empty(),
            Self::Not(x) => !x.eval(state),
            Self::Const(b) => *b,
        }
    }
}

struct Action {
    precond: StateToBool,
    postcond: StateToBool,
}
struct Spec {
    state_pred: Arc<StateToBool>,
    actions: Vec<Action>,
}

impl Spec {
    fn accepts_transition(
        &self,
        action_indexes: IndexSet,
        src: Arc<FactSet>,
        dest: Arc<FactSet>,
    ) -> bool {
        action_indexes.iter().all(|action_index| {
            let action = &self.actions[action_index];
            action.precond.eval(src.clone()) && action.postcond.eval(dest.clone())
        })
    }
    fn accepts_state(&self, fs: Arc<FactSet>) -> bool {
        self.state_pred.eval(fs)
    }
    fn update(&self, state: Arc<FactSet>, postcond: Arc<StateToState>) -> Arc<FactSet> {
        todo!()
    }
}

#[test]
fn zoop() {
    let mut spec = Spec { actions: vec![], state_pred: Arc::new(StateToBool::Const(true)) };
    let t = spec.accepts_state(Arc::new(FactSet { facts: Default::default() }));
    println!("{}", t);
}

// each state should have a FactSet (which I can query instantaneous properties)
// each transition has (1) a set of actions, (2) pre-condition, and (3) post-condition
// post-conditions
