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

type Delta = State;

struct Change {
    compute_delta: Box<dyn Fn(&mut dyn FnMut(FluentIndex) -> bool) -> Option<State>>,
    // compute_delta: Box<dyn Fn(&mut dyn Statelike) -> State>,
}
#[derive(Debug, Clone)]
struct SdVecSet<T: Ord> {
    // invariant: sorted, deduplicated
    vec: Vec<T>,
}
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Edge {
    from: ChangeDagIndex,
    to: ChangeDagIndex,
}
struct ChangeDag {
    verts: ChangeDagIndexSet, // set of Change identifiers
    edges: SdVecSet<Edge>,
}
struct TopSortIter<'a> {
    cd: &'a ChangeDag,
    // list elements U vert_mask = cd.verts
    vert_mask: IndexSet,
    list: Vec<ChangeDagIndex>,
}
////////////
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
impl<T: Ord> SdVecSet<T> {
    fn new(mut vec: Vec<T>) -> Self {
        vec.sort();
        vec.dedup();
        Self { vec }
    }
}
impl ChangeDag {
    fn edges_from(&self, from: ChangeDagIndex) -> &[Edge] {
        use core::cmp::Ordering::*;
        let left = self.edges.vec[..]
            .binary_search_by(|edge| if edge.from < from { Less } else { Greater })
            .unwrap_err();
        let right = left
            + self.edges.vec[left..]
                .binary_search_by(|edge| if edge.from <= from { Less } else { Greater })
                .unwrap_err();
        &self.edges.vec[left..right]
    }
}
impl<'a> TopSortIter<'a> {
    fn new(&'a mut self, cd: &'a ChangeDag) -> Self {
        Self { list: Vec::with_capacity(cd.verts.len()), vert_mask: cd.verts.clone(), cd }
    }
    fn fill_remaining(&mut self) {
        while !self.vert_mask.is_empty() {}
    }
    fn next(&mut self) -> Option<&[ChangeDagIndex]> {
        Some(&self.list)
    }
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
            if let Some(delta) = (self.compute_delta)(&mut funcy) {
                deltas.insert(delta);
            }
        }
        deltas
    }
}

#[test]
fn zorp() {
    let change_dag = ChangeDag {
        verts: [].into_iter().collect(), // yarp
        edges: SdVecSet::new(vec![
            Edge { from: 0, to: 0 }, // yeh
            Edge { from: 1, to: 1 }, // yeh
            Edge { from: 1, to: 2 }, // yeh
            Edge { from: 1, to: 3 }, // yeh
            Edge { from: 1, to: 5 }, // yeh
            Edge { from: 2, to: 2 }, // yeh
            Edge { from: 2, to: 5 }, // yeh
        ]),
    };
    dbg!(&change_dag.edges);
    dbg!(change_dag.edges_from(1));
    let c = Change {
        compute_delta: Box::new(|closure: &mut dyn FnMut(FluentIndex) -> bool| {
            let arr = [(0, !(closure)(0))];
            Some(arr.into_iter().collect())
        }),
    };
    let deltas = c.compute_deltas(Delta::default());
    println!("{:#?}", deltas);
}
