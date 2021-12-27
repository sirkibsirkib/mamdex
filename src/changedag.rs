use chunked_index_set::ChunkRead;
use core::alloc::Layout;

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

struct SquareBitMatrix {
    // Invariant A: Self::layout(self.row_bits) returns the owned, heap-allocated buffer pointed to with ptr if any.
    ptr: *mut usize,
    row_bits: usize,
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
    /// Graph is a DAG -> this method succeeds.
    /// Why? with any subset of verts removed, there are 1+ min elements!
    fn min_fill_remaining(&mut self) {
        while !self.vert_mask.is_empty() {
            let ci = self.remove_min().expect("cycle detected!");
            self.list.push(ci);
        }
        assert_eq!(self.list.len(), self.cd.verts.len());
    }
    fn next(&mut self) -> Option<&[ChangeIndex]> {
        if self.list.is_empty() {
            // first time! min_fill whole thing and return!
        } else {
            // not first time! need to advance past what I've got buffered
            loop {
                let ci = self.list.pop()?;
                self.vert_mask.insert(ci);
                if let Some(larger) = self.remove_min_larger_than(ci) {
                    self.list.push(larger);
                    break;
                }
            }
        }
        self.min_fill_remaining();
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
impl SquareBitMatrix {
    const WORD_BITS: usize = usize::BITS as usize;
    const WORD_BYTES: usize = Self::WORD_BITS / 8;
    fn words_per_row(row_bits: usize) -> usize {
        (row_bits + Self::WORD_BITS - 1) / Self::WORD_BITS
    }
    fn layout(row_bits: usize) -> Option<Layout> {
        if row_bits == 0 {
            None
        } else {
            let words = Self::words_per_row(row_bits) * row_bits * 8;
            unsafe {
                // safe! this layout is (1) aligned, and (2) nonzero size
                Some(std::alloc::Layout::from_size_align_unchecked(words, Self::WORD_BYTES))
            }
        }
    }
    ///////
    pub fn new(row_bits: usize) -> Self {
        let ptr = match Self::layout(row_bits) {
            None => core::ptr::null_mut(),
            Some(layout) => unsafe {
                println!("{:?}", layout);
                // safe! No documented failure condition
                std::alloc::alloc_zeroed(layout) as *mut usize
            },
        };
        // Invariant A establised
        Self { row_bits, ptr }
    }
    fn in_bounds(&self, bit_coord: [usize; 2]) -> bool {
        bit_coord.into_iter().all(|x| x < self.row_bits)
    }
    fn bit_address(&self, bit_coord: [usize; 2]) -> Option<[usize; 2]> {
        if self.in_bounds(bit_coord) {
            let [x, y] = bit_coord;
            let stride = Self::words_per_row(self.row_bits);
            let idx_of_word = y * stride + (x / Self::WORD_BITS);
            let idx_in_word = x % Self::WORD_BITS;
            Some([idx_of_word, idx_in_word])
        } else {
            None
        }
    }
    fn get_word_mut(&mut self, bit_coord: [usize; 2]) -> Option<(&mut usize, usize)> {
        let [idx_of_word, idx_in_word] = self.bit_address(bit_coord)?;
        let word = unsafe {
            // Stays within allocated bounds :- Invariant A
            let word_ptr = self.ptr.add(idx_of_word);
            // Mutable access OK :- Invariant A
            &mut *word_ptr
        };
        Some((word, idx_in_word))
    }
    fn get_word(&self, bit_coord: [usize; 2]) -> Option<(usize, usize)> {
        let [idx_of_word, idx_in_word] = self.bit_address(bit_coord)?;
        let word = unsafe {
            // Stays within allocated bounds :- Invariant A
            let word_ptr = self.ptr.add(idx_of_word);
            // Mutable access OK :- Invariant A
            *word_ptr
        };
        Some((word, idx_in_word))
    }
    pub fn insert(&mut self, bit_coord: [usize; 2]) -> Option<bool> {
        let (word, idx_in_word) = self.get_word_mut(bit_coord)?;
        let word_was = *word;
        *word |= 1 << idx_in_word;
        Some(*word != word_was)
    }
    pub fn remove(&mut self, bit_coord: [usize; 2]) -> Option<bool> {
        let (word, idx_in_word) = self.get_word_mut(bit_coord)?;
        let word_was = *word;
        *word &= !(1 << idx_in_word);
        Some(*word != word_was)
    }
    pub fn contains(&self, bit_coord: [usize; 2]) -> Option<bool> {
        let (word, idx_in_word) = self.get_word(bit_coord)?;
        Some((word & 1 << idx_in_word) != 0)
    }
    pub fn transitively_close(&mut self) {
        let stride = Self::words_per_row(self.row_bits);
        loop {
            let mut was_change = false;
            let mut from_row_ptr = self.ptr;
            // for each from-row
            unsafe {
                // lots of pointer manipulation but all in bounds
                for _ in 0..self.row_bits {
                    let mut to_row_ptr = self.ptr;
                    // for each to-row
                    for _ in 0..self.row_bits {
                        if from_row_ptr == to_row_ptr {
                            continue;
                        }
                        let mut f = from_row_ptr;
                        let mut t = to_row_ptr;
                        for _ in 0..stride {
                            //
                            let t_old = *t;
                            *t |= *f;
                            if *t != t_old {
                                was_change = true
                            }

                            f = f.add(1);
                            t = t.add(1);
                        }
                        to_row_ptr = to_row_ptr.add(stride);
                    }
                    from_row_ptr = from_row_ptr.add(stride);
                }
            }
            if !was_change {
                break;
            }
        }
    }

    pub fn any_self_loop(&self) -> Option<usize> {
        for i in 0..self.row_bits {
            if Some(true) == self.contains([i, i]) {
                return Some(i);
            }
        }
        None
    }
    pub fn print(&self) {
        for y in 0..self.row_bits {
            for x in 0..self.row_bits {
                let c = if self.contains([x, y]) == Some(true) { "1" } else { "0" };
                print!("{}", c);
            }
            print!("\n");
        }
    }
}

impl Drop for SquareBitMatrix {
    fn drop(&mut self) {
        if let Some(layout) = Self::layout(self.row_bits) {
            unsafe {
                // safe! :- Invariant A
                std::alloc::dealloc(self.ptr as *mut u8, layout)
            }
        }
    }
}

///////////////
#[test]
fn mat() {
    let mut m = SquareBitMatrix::new(5);
    m.insert([0, 1]);
    m.insert([1, 2]);
    m.insert([2, 3]);
    m.print();
    println!();
    m.transitively_close();
    m.print();
}

#[test]
fn topological_sort() {
    let change_dag = ChangeDag {
        verts: [0, 1, 2, 3, 4].into_iter().collect(), // yarp
        edges: SdVecSet::new(vec![
            Edge { from: 0, to: 1 }, // yeh
            Edge { from: 0, to: 2 }, // yeh
            Edge { from: 1, to: 3 }, // yeh
            Edge { from: 2, to: 3 }, // yeh
            Edge { from: 2, to: 4 }, // yeh
        ]),
    };
    let mut tsi = TopSortIter::new(&change_dag);
    while let Some(x) = tsi.next() {
        println!("{:?}", x);
    }
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
