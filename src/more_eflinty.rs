use core::num::NonZeroU32;
use std::collections::HashSet;

use std::sync::Arc;

// type IndexSet = chunked_index_set::IndexSet<2>;

/*
let's take the union approach; a fact is true iff its postulated OR there's 1+ ways to derive it
the essence of a state is the set of postulated facts
we don't need non-boolean predicates; they are just partial functions we can worry about later
duties are just facts marked as "necessary for satisfaction". let's worry about that later
*/

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Atom {
    data: NonZeroU32,
}
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Fact {
    atoms: Arc<[Atom]>,
}
#[derive(Debug, Clone, Hash)]
struct FactPattern {
    maybe_atoms: Arc<[Option<Atom>]>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Label {
    data: u32,
}
trait FactState: Clone {
    type QueryIter: Iterator<Item = Fact>;
    fn fact_query(&self, fact: &Fact) -> bool;
    fn pattern_query(&self, pattern: &FactPattern) -> Self::QueryIter;
    fn create(&mut self, fact: &Fact);
    fn terminate(&mut self, fact: &Fact);
}

trait Specification<S: FactState> {
    fn duty_compliant(&self, state: &S) -> bool;
    fn action_compliant(&self, source: &S, label: Label, dest: &S) -> bool;
    fn try_act(&self, source: &S, label: Label) -> Option<S>;
}

#[derive(Debug, Clone)]
struct FactSet {
    hold: HashSet<Fact>,
}
impl FactPattern {
    fn matches(&self, fact: &Fact) -> bool {
        self.maybe_atoms.len() == fact.atoms.len()
            && self.maybe_atoms.iter().zip(fact.atoms.iter()).all(|(pat, at)| match (pat, at) {
                (None, _) => true,
                (Some(pat), at) => pat == at,
            })
    }
}
impl FactState for FactSet {
    type QueryIter = Cloned<Filter<hash_set::Iter<Fact>>;
    fn fact_query(&self, fact: &Fact) -> bool {
        self.hold.contains(fact)
    }
    fn pattern_query(&self, pattern: &FactPattern) -> Self::QueryIter {
        self.hold.iter().filter(|fact| pattern.matches(fact)).cloned()
    }
    fn create(&mut self, fact: &Fact) {
        self.hold.insert(fact.clone());
    }
    fn terminate(&mut self, fact: &Fact) {
        self.hold.remove(fact);
    }
}

struct Agreement {}
// impl<S: FactState> Specification<S>
