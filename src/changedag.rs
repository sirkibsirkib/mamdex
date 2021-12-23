use chunked_index_set::ChunkRead;

use std::collections::HashSet;

use core::cmp::Ordering::{self, *};

type IndexSet = chunked_index_set::IndexSet<1>;
type FluentIndexSet = IndexSet;
type FluentIndex = usize;
type ChangeIndexSet = IndexSet;
type ChangeIndex = usize;

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
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Edge {
    to: ChangeIndex,
    from: ChangeIndex,
}
struct ChangeDag {
    verts: ChangeIndexSet, // set of Change identifiers
    edges: SdVecSet<Edge>,
}
struct TopSortIter<'a> {
    cd: &'a ChangeDag,
    // list elements U vert_mask = cd.verts
    vert_mask: ChangeIndexSet,
    list: Vec<ChangeIndex>,
}
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
    fn edges_to(&self, to: ChangeIndex) -> &[Edge] {
        let left = self.edges.vec[..]
            .binary_search_by(|edge| if edge.to < to { Less } else { Greater })
            .unwrap_err();
        let right = left
            + self.edges.vec[left..]
                .binary_search_by(|edge| if edge.to <= to { Less } else { Greater })
                .unwrap_err();
        &self.edges.vec[left..right]
    }
}
impl<'a> TopSortIter<'a> {
    fn new(cd: &'a ChangeDag) -> Self {
        Self { list: Vec::with_capacity(cd.verts.len()), vert_mask: cd.verts.clone(), cd }
    }
    fn vec_truncate_at(&mut self, index: usize) {
        for &ci in self.list[index..].iter() {
            self.vert_mask.insert(ci);
        }
        self.list.truncate(index);
    }
    fn remove_min_larger_than(&mut self, than: ChangeIndex) -> Option<ChangeIndex> {
        for ci in self.vert_mask.iter().skip_while(|&ci| ci <= than) {
            // return this ci if all its incoming edges are from REMOVED verts
            if self.cd.edges_to(ci).iter().all(|edge| !self.vert_mask.contains(edge.from)) {
                self.vert_mask.remove(ci);
                return Some(ci);
            }
        }
        None
    }
    fn remove_min(&mut self) -> Option<ChangeIndex> {
        for ci in self.vert_mask.iter() {
            // return this ci if all its incoming edges are from REMOVED verts
            if self.cd.edges_to(ci).iter().all(|edge| !self.vert_mask.contains(edge.from)) {
                self.vert_mask.remove(ci);
                return Some(ci);
            }
        }
        None
    }
    fn fill_remaining(&mut self) {
        while !self.vert_mask.is_empty() {
            let ci = self.remove_min().expect("cycle detected!");
            self.list.push(ci);
        }
        assert_eq!(self.list.len(), self.cd.verts.len());
    }
    fn next(&mut self) -> Option<&[ChangeIndex]> {
        if self.list.is_empty() {
            // first time! find smallest element and return it
        } else {
            // not first time! need to advance past what I've got buffered
            loop {
                let ci = self.list.pop()?;
                self.vert_mask.insert(ci);
                if let Some(larger) = self.remove_min_larger_than(ci) {
                    println!("{:?} {:?}", ci, larger);
                    self.list.push(larger);
                    break;
                }
            }
        }
        self.fill_remaining();
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
fn topological_sort() {
    let change_dag = ChangeDag {
        verts: [0, 1, 2, 3].into_iter().collect(), // yarp
        edges: SdVecSet::new(vec![
            Edge { from: 0, to: 1 }, // yeh
            Edge { from: 1, to: 2 }, // yeh
        ]),
    };
    let mut tsi = TopSortIter::new(&change_dag);
    while let Some(x) = tsi.next() {
        println!("{:?}", x);
    }
    // dbg!(&change_dag.edges);
    // dbg!(change_dag.edges_to(1));
    // println!("{:#?}", deltas);
}

#[test]
fn change_compute() {
    let c = Change {
        compute_delta: Box::new(|closure: &mut dyn FnMut(FluentIndex) -> bool| {
            let arr = [(0, !(closure)(0))];
            Some(arr.into_iter().collect())
        }),
    };
    let _deltas = c.compute_deltas(Delta::default());
}
