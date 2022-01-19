use core::ops::Range;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Event {
    SetOwner { new_owner_name: bool },
    BecomeFriends { a: bool, b: bool },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Signature {
    name: &'static str,
}

struct EventGraph {
    happen: HashSet<Event>,
    before: HashSet<[Event; 2]>,
}
struct PartialEventGraph {
    depend: HashSet<Event>,
    event_graph: EventGraph,
}
struct SignedEventGraph {
    signatures: Vec<Signature>,
    partial_event_graph: PartialEventGraph,
}

#[derive(Clone, Debug, Default)]
struct Situation {
    truth: HashMap<Fact, bool>,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Fact {
    pub bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct FactPattern {
    fact: Fact,
    mask: u32,
}

fn pair_copy<A: Copy, B: Copy>((&a, &b): (&A, &B)) -> (A, B) {
    (a, b)
}

//////////////
impl FactPattern {
    const fn from_slice(bits: u32, bit_range: Range<u8>) -> Self {
        let mask = bit_mask(range_copy(&bit_range));
        Self { fact: Fact { bits: (bits << bit_range.start) & mask }, mask }
    }
}
const fn bit_mask(range: Range<u8>) -> u32 {
    let from_start = !0 << range.start;
    let to_end = !0 << range.end;
    from_start & to_end
}
const fn range_copy(range: &Range<u8>) -> Range<u8> {
    range.start..range.end
}
impl Situation {
    fn try_union(&self, other: &Self) -> Option<Self> {
        let mut new = self.clone();
        new.consistent_insert_all(other.truth.iter().map(pair_copy))?;
        Some(new)
    }
    fn consistent_insert_all(
        &mut self,
        it: impl Iterator<Item = (Fact, bool)> + Clone,
    ) -> Option<()> {
        // check for inconsistencies
        for (fact, value) in it.clone() {
            if self.truth.get(&fact).copied() == Some(!value) {
                return None;
            }
        }
        for (fact, value) in it.clone() {
            self.truth.insert(fact, value);
        }
        Some(())
    }
    pub fn insert(&mut self, fact: Fact, value: bool) -> Option<bool> {
        self.truth.insert(fact, value)
    }
    pub fn query(
        &self,
        fact_pattern: FactPattern,
    ) -> impl Iterator<Item = (Fact, bool)> + '_ + Clone {
        self.truth
            .iter()
            .filter(move |(fact, _value)| fact.matches_pattern(fact_pattern))
            .map(pair_copy)
    }
    pub fn try_delta(&self, event: Event) -> Option<Self> {
        let mut delta = Situation::default();
        match event {
            Event::SetOwner { new_owner_name } => {
                delta.truth.extend(
                    self.query(FactPattern::from_slice(0b0, 0..1))
                        .map(|(fact, _value)| (fact, false)),
                );
                let fact = Fact::default()
                    .with(FactPattern::from_slice(0b0, 0..1))
                    .with(FactPattern::from_slice(if new_owner_name { 1 } else { 0 }, 1..2));
                delta.insert(fact, true);
            }
            Event::BecomeFriends { a, b } => {
                let fact = Fact::default()
                    .with(FactPattern::from_slice(0b1, 0..1))
                    .with(FactPattern::from_slice(if a { 1 } else { 0 }, 1..2))
                    .with(FactPattern::from_slice(if b { 1 } else { 0 }, 2..3));
                delta.insert(fact, true);
            }
        }
        Some(delta)
    }
}

impl EventGraph {
    fn transitively_close_happened(&mut self) {
        'outer: loop {
            for &[from, via] in self.before.iter() {
                if !self.happen.contains(&from) || !self.happen.contains(&via) {
                    continue;
                }
                for &to in self.happen.iter() {
                    if self.before.contains(&[via, to]) && !self.before.contains(&[from, to]) {
                        self.before.insert([from, to]);
                        continue 'outer;
                    }
                }
            }
        }
    }
}
impl Fact {
    pub const fn matches_pattern(self, fact_pattern: FactPattern) -> bool {
        self.bits & fact_pattern.mask == fact_pattern.fact.bits
    }
    pub const fn with(mut self, fact_pattern: FactPattern) -> Self {
        self.bits &= !fact_pattern.mask;
        self.bits |= fact_pattern.fact.bits;
        self
    }
    pub const fn read(self, bit_range: Range<u8>) -> u32 {
        self.bits & bit_mask(range_copy(&bit_range)) << bit_range.start
    }
}
