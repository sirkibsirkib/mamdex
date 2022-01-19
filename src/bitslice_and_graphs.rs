use core::hash::Hash;
use core::ops::BitAnd;
use core::ops::BitOrAssign;
use core::ops::Range;
use enum_map::{enum_map, Enum, EnumMap};
use maplit::hashset;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

struct HeapPermute<'a, T> {
    arr: &'a mut [T],
    i: usize,
    n: usize,
    c: Vec<usize>,
}
impl<'a, T> HeapPermute<'a, T> {
    /// https://en.wikipedia.org/wiki/Heap%27s_algorithm
    fn new(arr: &'a mut [T]) -> Self {
        assert!(arr.len() < usize::MAX);
        Self { n: usize::MAX, c: std::iter::repeat(0).take(arr.len()).collect(), i: 0, arr }
    }
    fn next(&mut self) -> Option<&[T]> {
        if self.n == usize::MAX {
            self.n = self.arr.len();
            return Some(self.arr);
        }
        while self.i < self.n {
            if self.c[self.i] < self.i {
                if self.i % 2 == 0 {
                    self.arr.swap(0, self.i);
                } else {
                    self.arr.swap(self.c[self.i], self.i);
                }
                self.c[self.i] += 1;
                self.i = 0;
                return Some(self.arr);
            } else {
                self.c[self.i] = 0;
                self.i += 1;
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct EventInstance {
    event: Event,
    index: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Event {
    SetOwner { owner: bool },
    BecomeFriends { a: bool, b: bool },
}

// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// struct Signature {
//     name: &'static str,
// }

struct ClosedOrder {
    before: HashSet<[EventInstance; 2]>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EventGraph {
    happen: HashSet<EventInstance>,
    before: HashSet<[EventInstance; 2]>,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PartialEventGraph {
    depend: HashSet<EventInstance>,
    event_graph: EventGraph,
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct SignedEventGraph {
//     signatures: HashSet<Signature>,
//     partial_event_graph: PartialEventGraph,
// }

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

#[derive(Enum, Copy, Clone, Debug, Serialize, Deserialize)]
enum Agent {
    Amy,
    Bob,
    Dan,
}

#[derive(Debug, Serialize, Deserialize)]
enum Task {
    AddToHistory(EventGraph),
    PrintHistory,
    CurrentSituations,
}

#[derive(Debug, Serialize, Deserialize)]
struct Input {
    agent: Agent,
    task: Task,
}

//////////////
impl<'a> BitAnd<&'a Self> for Situation {
    type Output = Option<Situation>;
    fn bitand(self, rhs: &Self) -> Option<Situation> {
        self.bitand(rhs.truth.iter().map(pair_copy))
    }
}
impl<I: Iterator<Item = (Fact, bool)> + Clone> BitAnd<I> for Situation {
    type Output = Option<Situation>;
    fn bitand(mut self, rhs: I) -> Option<Situation> {
        for (fact, value) in rhs {
            let was = self.truth.insert(fact, value);
            if was != Some(value) {
                return None;
            }
        }
        Some(self)
    }
}
impl<'a> BitOrAssign<&'a EventGraph> for EventGraph {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.happen.extend(rhs.happen.iter().copied());
        self.before.extend(rhs.before.iter().copied());
    }
}
impl<'a> BitOrAssign<&'a PartialEventGraph> for PartialEventGraph {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.depend.extend(rhs.depend.iter().copied());
        self.event_graph |= &rhs.event_graph;
    }
}
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
            Event::SetOwner { owner } => {
                delta.truth.extend(
                    self.query(FactPattern::from_slice(0b0, 0..1))
                        .map(|(fact, _value)| (fact, false)),
                );
                let fact = Fact::default()
                    .with(FactPattern::from_slice(0b0, 0..1))
                    .with(FactPattern::from_slice(if owner { 1 } else { 0 }, 1..2));
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

impl ClosedOrder {
    fn respected_by(&self, arr: &[EventInstance]) -> bool {
        for window in arr.windows(2) {
            if let &[a, b] = window {
                if self.before.contains(&[b, a]) {
                    return false;
                }
            }
        }
        true
    }
    fn take_cycle(&self, happen: &HashSet<EventInstance>) -> Option<EventInstance> {
        happen.iter().copied().find(|&x| self.before.contains(&[x, x]))
    }
}
impl EventGraph {
    fn closed_before(&self) -> ClosedOrder {
        ClosedOrder { before: Self::transitively_close_before(&self.happen, self.before.clone()) }
    }
    fn transitively_close_before(
        happen: &HashSet<EventInstance>,
        mut before: HashSet<[EventInstance; 2]>,
    ) -> HashSet<[EventInstance; 2]> {
        'outer: loop {
            for &[from, via] in before.iter() {
                if !happen.contains(&from) || !happen.contains(&via) {
                    continue;
                }
                for &to in happen.iter() {
                    if before.contains(&[via, to]) && !before.contains(&[from, to]) {
                        before.insert([from, to]);
                        continue 'outer;
                    }
                }
            }
            break before;
        }
    }
}
impl PartialEventGraph {
    fn is_complete(&self) -> bool {
        self.depend.is_subset(&self.event_graph.happen)
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

pub fn run2() {
    let initial = EventGraph::default();
    let mut agent_histories: EnumMap<Agent, EventGraph> = enum_map! {
        Agent::Amy => initial.clone(),
        Agent::Bob => initial.clone(),
        Agent::Dan => initial.clone(),
    };
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();

    let mut buffer = String::new();
    loop {
        use std::io::BufRead;
        stdin_lock.read_line(&mut buffer).unwrap();
        let got = ron::de::from_str::<Input>(&buffer);
        println!("Got: {:#?}", &got);
        buffer.clear();
        match got {
            Ok(Input { agent, task: Task::AddToHistory(event_graph) }) => {
                agent_histories[agent] |= &event_graph;
            }

            Ok(Input { agent, task: Task::PrintHistory }) => {
                println!("{:#?}", &agent_histories[agent]);
            }
            Ok(Input { agent, task: Task::CurrentSituations }) => {
                let h = &agent_histories[agent];
                let mut happen_vec: Vec<_> = h.happen.iter().copied().collect();
                let closed_before = h.closed_before();
                let mut hp = HeapPermute::new(&mut happen_vec);
                while let Some(arr) = hp.next() {
                    if closed_before.respected_by(arr) {
                        println!("arr {:?}", arr);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run() {
    let [e0, e1, e2] = [
        EventInstance { event: Event::SetOwner { owner: false }, index: 0 }, // noice
        EventInstance { event: Event::SetOwner { owner: false }, index: 1 }, // noice
        EventInstance { event: Event::SetOwner { owner: false }, index: 2 }, // noice
    ];
    let history = EventGraph { happen: hashset! {e0,e1,e2}, before: hashset! {[e0,e1], [e0,e2]} };
    let closed_before = history.closed_before();
    let mut arr: Vec<_> = history.happen.iter().copied().collect();
    let mut hp = HeapPermute::new(&mut arr);
    while let Some(arr) = hp.next() {
        if closed_before.respected_by(arr) {
            println!("arr {:#?}", arr);
        }
    }
}

/*
Input(agent:Amy, task:AddToHistory(EventGraph ( happen:[SetOwner(owner:false)], before:[])))
Input(agent:Amy, task:PrintHistory)
*/
