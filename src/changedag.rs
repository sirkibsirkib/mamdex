use chunked_index_set::ChunkRead;

use std::collections::HashSet;

type IndexSet = chunked_index_set::IndexSet<1>;
type FluentIndexSet = IndexSet;
type FluentIndex = usize;
type ChangeDagIndexSet = IndexSet;
type ChangeDagIndex = usize;

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
struct State {
    // acts as FluentIndex -> Option<bool>
    known: FluentIndexSet,
    holds: FluentIndexSet,
}
// impl PartialOrd for State {
//     fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(rhs))
//     }
// }
// impl Ord for State {
//     /// lexicographic chunk_list_cmp ordering of (known, holds).
//     fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
//         use core::cmp::Ordering::Equal;
//         match self.known.chunk_list_cmp(&rhs.known) {
//             Equal => self.holds.chunk_list_cmp(&rhs.holds),
//             x => x,
//         }
//     }
// }

// trait Precondition {
// 	fn query_fluent(&mut self, fluent: FluentIndex) -> bool;
// }

type Delta = State;

struct Change {
    compute_delta: Box<dyn Fn(&mut dyn FnMut(FluentIndex) -> bool) -> State>,
    // compute_delta: Box<dyn Fn(&mut dyn Statelike) -> State>,
}

//////////

impl FromIterator<(FluentIndex, bool)> for State {
    fn from_iter<T: IntoIterator<Item = (FluentIndex, bool)>>(iter: T) -> Self {
        let mut me = Self::default();
        for (fluent, value) in iter.into_iter() {
            me.known.insert(fluent);
            me.holds.make_contains(fluent, value);
        }
        me
    }
}
impl State {
    fn test(&self, fluent: FluentIndex) -> Option<bool> {
        if self.known.contains(fluent) {
            Some(self.holds.contains(fluent))
        } else {
            None
        }
    }
    fn speculate(&mut self, fluent: FluentIndex) -> (bool, Option<Self>) {
        if self.known.insert(fluent) {
            let mut me_but_holds = self.clone();
            me_but_holds.holds.insert(fluent);
            self.holds.remove(fluent);
            (false, Some(me_but_holds))
        } else {
            (self.holds.contains(fluent), None)
        }
    }
}

struct Edge {
    before: ChangeDagIndex,
    after: ChangeDagIndex,
}
struct ChangeDag {
    verts: ChangeDagIndexSet, // set of Change identifiers
    edges: Vec<Edge>,
}

struct TopSortIter<'a> {
    cd: &'a ChangeDag,
    list: Vec<ChangeDagIndex>,
}

impl Change {
    fn compute_deltas(&self, precond: State) -> HashSet<State> {
        let mut preconds_todo: Vec<State> = std::iter::once(precond).collect();
        let mut deltas: HashSet<State> = Default::default();
        while let Some(mut precond) = preconds_todo.pop() {
            let mut funcy = |fluent| {
                let (holds, maybe_branch) = precond.speculate(fluent);
                if let Some(branch) = maybe_branch {
                    preconds_todo.push(branch);
                }
                holds
            };
            deltas.insert((self.compute_delta)(&mut funcy));
        }
        deltas
    }
}

#[test]
fn zorp() {
    let c = Change {
        compute_delta: Box::new(|closure: &mut dyn FnMut(FluentIndex) -> bool| {
            let arr = [(0, !(closure)(0))];
            arr.into_iter().collect()
        }),
    };
    let deltas = c.compute_deltas(Delta::default());
    println!("{:#?}", deltas);
}
