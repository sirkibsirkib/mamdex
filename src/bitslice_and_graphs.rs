use core::fmt::Debug;
use core::hash::Hash;
use core::ops::BitAnd;
use core::ops::BitOrAssign;
use core::ops::Range;
use enum_map::{enum_map, Enum, EnumMap};
use maplit::hashset;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

#[derive(Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Situation {
    truth: BTreeMap<Fact, bool>,
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Fact {
    pub bits: u32,
}

#[derive(Debug)]
enum FactHr {
    Owner { owner: bool },
    Friend { a: bool, b: bool },
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
    AgentHistoryAdd { agent: Agent, graph: EventGraph },
    AgentHistoryPrint { agent: Agent },
    AgentDestinationsPrint { agent: Agent },
    GlobalHistoryPrint,
    GlobalDestinationsPrint,
}
trait Compose<T> {
    fn compose(&mut self, rhs: &T);
    fn composed(mut self, rhs: &T) -> Self
    where
        Self: Sized,
    {
        self.compose(rhs);
        self
    }
}
//////////////
impl Into<FactHr> for Fact {
    fn into(self) -> FactHr {
        match self.read(0..1) {
            0 => FactHr::Owner { owner: self.read(1..2) == 1 },
            1 => FactHr::Friend { a: self.read(1..2) == 1, b: self.read(2..3) == 1 },
            _ => unreachable!(),
        }
    }
}
impl Into<Fact> for FactHr {
    fn into(self) -> Fact {
        match self {
            FactHr::Owner { owner } => Fact::default()
                .with(FactPattern::from_slice(0b0, 0..1))
                .with(FactPattern::from_slice(if owner { 1 } else { 0 }, 1..2)),
            FactHr::Friend { a, b } => Fact::default()
                .with(FactPattern::from_slice(0b1, 0..1))
                .with(FactPattern::from_slice(if a { 1 } else { 0 }, 1..2))
                .with(FactPattern::from_slice(if b { 1 } else { 0 }, 2..3)),
        }
    }
}
// impl<'a> BitAnd<&'a Self> for Situation {
//     type Output = Option<Situation>;
//     fn bitand(self, rhs: &Self) -> Option<Situation> {
//         self.bitand(rhs.truth.iter().map(pair_copy))
//     }
// }
// impl<I: Iterator<Item = (Fact, bool)> + Clone> BitAnd<I> for Situation {
//     type Output = Option<Situation>;
//     fn bitand(mut self, rhs: I) -> Option<Situation> {
//         for (fact, value) in rhs {
//             let was = self.truth.insert(fact, value);
//             if was != Some(value) {
//                 return None;
//             }
//         }
//         Some(self)
//     }
// }
// impl<'a> BitOrAssign<&'a EventGraph> for EventGraph {
//     fn bitor_assign(&mut self, rhs: &Self) {
//         self.happen.extend(rhs.happen.iter().copied());
//         self.before.extend(rhs.before.iter().copied());
//     }
// }
// impl<'a> BitOrAssign<&'a PartialEventGraph> for PartialEventGraph {
//     fn bitor_assign(&mut self, rhs: &Self) {
//         self.depend.extend(rhs.depend.iter().copied());
//         self.event_graph |= &rhs.event_graph;
//     }
// }
impl FactPattern {
    fn from_slice(bits: u32, bit_range: Range<u8>) -> Self {
        let mask = bit_mask(range_copy(&bit_range));
        println!("mask {:b}", mask);
        Self { fact: Fact { bits: (bits << bit_range.start) & mask }, mask }
    }
}
const fn bit_mask(range: Range<u8>) -> u32 {
    let from_start = !0 << range.start;
    let to_end = !0 << range.end;
    !(from_start & to_end)
}
const fn range_copy(range: &Range<u8>) -> Range<u8> {
    range.start..range.end
}
impl Situation {
    pub fn update(&mut self, rhs: &Self) {
        for (&fact, &value) in rhs.truth.iter() {
            self.truth.insert(fact, value);
        }
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
        println!("delta for event {:?}", event);
        match event {
            Event::SetOwner { owner } => {
                delta.truth.extend(
                    self.query(FactPattern::from_slice(0b0, 0..1))
                        .map(|(fact, _value)| (fact, false)),
                );
                delta.insert(FactHr::Owner { owner }.into(), true);
            }
            Event::BecomeFriends { a, b } => {
                delta.insert(FactHr::Friend { a, b }.into(), true);
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
impl Compose<Self> for EventGraph {
    fn compose(&mut self, rhs: &Self) {
        self.happen.extend(rhs.happen.iter().copied());
        self.before.extend(rhs.before.iter().copied());
    }
}
impl EventGraph {
    fn closed_before(&self) -> ClosedOrder {
        ClosedOrder { before: Self::transitively_close_before(&self.happen, self.before.clone()) }
    }
    fn destinations(
        &self,
        initial_situation: &Situation,
    ) -> BTreeMap<Situation, Vec<EventInstance>> {
        let mut eq_classes = BTreeMap::<Situation, Vec<EventInstance>>::default();
        let closed_before = self.closed_before();
        let mut arr: Vec<_> = self.happen.iter().copied().collect();
        let mut hp = HeapPermute::new(&mut arr);
        while let Some(arr) = hp.next() {
            if closed_before.respected_by(arr) {
                // println!("arr {:#?}", arr);
                let mut sit = initial_situation.clone();
                for ei in arr {
                    let delta = sit.try_delta(ei.event).unwrap();
                    // println!("delta now {:?}", &delta);
                    sit.update(&delta);
                    // println!("sit now {:?}", &sit);
                }
                // println!("END SIT {:#?}", &sit);
                if !eq_classes.contains_key(&sit) {
                    eq_classes.insert(sit, arr.to_vec());
                }
            }
        }
        eq_classes
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
impl Debug for Situation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_map()
            .entries(self.truth.iter().map(|(&k, v)| (Into::<FactHr>::into(k), v)))
            .finish()
    }
}
impl Compose<Self> for PartialEventGraph {
    fn compose(&mut self, rhs: &Self) {
        self.depend.extend(rhs.depend.iter().copied());
        self.event_graph.compose(&rhs.event_graph);
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
        (self.bits & bit_mask(range_copy(&bit_range))) >> bit_range.start
    }
}

pub fn repl() {
    let initial_situation = Situation::default();
    let initial_history = EventGraph::default();
    let mut agent_histories: EnumMap<Agent, EventGraph> = enum_map! {
        Agent::Amy => initial_history.clone(),
        Agent::Bob => initial_history.clone(),
        Agent::Dan => initial_history.clone(),
    };
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();

    let mut buffer = String::new();
    loop {
        use std::io::BufRead;
        stdin_lock.read_line(&mut buffer).unwrap();
        let got = ron::de::from_str::<Task>(&buffer);
        println!("Got: {:#?}", &got);
        buffer.clear();
        match got {
            Ok(Task::AgentHistoryAdd { agent, graph }) => agent_histories[agent].compose(&graph),
            Ok(Task::AgentHistoryPrint { agent }) => println!("{:#?}", &agent_histories[agent]),
            Ok(Task::AgentDestinationsPrint { agent }) => {
                let destinations = agent_histories[agent].destinations(&initial_situation);
                println!("{:#?}", &destinations);
            }
            Ok(Task::GlobalHistoryPrint) => {
                let global = agent_histories
                    .values()
                    .fold(EventGraph::default(), |global, local| global.composed(local));
                println!("{:#?}", &global);
            }
            Ok(Task::GlobalDestinationsPrint) => {
                let global = agent_histories
                    .values()
                    .fold(EventGraph::default(), |global, local| global.composed(local));
                let destinations = global.destinations(&initial_situation);
                println!("{:#?}", &destinations);
            }
            Err(_) => {}
        }
    }
}
