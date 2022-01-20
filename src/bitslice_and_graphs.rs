use core::{fmt::Debug, hash::Hash, ops::Range};
use enum_map::{enum_map, Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

const KIND_BITS_LEN: u8 = 2;
const KIND_METAS: &'static [FactKindMeta] = &[
    FactKindMeta {
        kind_name: "owner",
        field_metas: &[FactFieldMeta { field_name: "owner", bits_len: 2 }],
    },
    FactKindMeta {
        kind_name: "friend",
        field_metas: &[
            FactFieldMeta { field_name: "a", bits_len: 2 },
            FactFieldMeta { field_name: "b", bits_len: 2 },
        ],
    },
];
struct FactFieldMeta {
    field_name: &'static str,
    bits_len: u8,
}
#[derive(Clone, Copy)]
struct FactKindMeta {
    kind_name: &'static str,
    field_metas: &'static [FactFieldMeta],
}
struct FactHr(Fact);
struct HeapPermute<'a, T> {
    arr: &'a mut [T],
    i: usize,
    n: usize,
    c: Vec<usize>,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct EventInstance {
    event: Event,
    index: u32,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Event {
    SetOwner { owner: u32 },
    BecomeFriends { a: u32, b: u32 },
}
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
#[derive(Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Situation {
    truth: BTreeMap<Fact, bool>,
}
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
struct ReplState {
    initial_situation: Situation,
    agent_histories: EnumMap<Agent, EventGraph>,
}

//////////////////////////////////////////////////////

impl Debug for FactHr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let fkm = KIND_METAS[self.0.read(0..KIND_BITS_LEN) as usize];
        let mut ds = f.debug_struct("FactHr");
        let mut offset = KIND_BITS_LEN;
        ds.field("kind", &fkm.kind_name);
        for field_meta in fkm.field_metas {
            let new_offset = offset + field_meta.bits_len;
            ds.field(field_meta.field_name, &self.0.read(offset..new_offset));
            offset = new_offset;
        }
        ds.finish()
    }
}
impl Fact {
    fn pack(kind_idx: u8, field_bits: &[u32]) -> Self {
        let fkm = KIND_METAS[kind_idx as usize];
        let mut fact = Self::default();
        fact.write(FactPattern::from_bit_slice(kind_idx as u32, 0..KIND_BITS_LEN));
        let mut offset = KIND_BITS_LEN;
        for (&field_bits, field_meta) in field_bits.iter().zip(fkm.field_metas) {
            let new_offset = offset + field_meta.bits_len;
            fact.write(FactPattern::from_bit_slice(field_bits as u32, offset..new_offset));
            offset = new_offset;
        }
        fact
    }
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
impl FactPattern {
    fn from_bit_slice(bits: u32, bit_range: Range<u8>) -> Self {
        let mask = bit_mask(range_copy(&bit_range));
        // println!("mask {:b}", mask);
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
        // println!("delta for event {:?}", event);
        match event {
            Event::SetOwner { owner } => {
                delta.truth.extend(
                    self.query(FactPattern::from_bit_slice(0b0, 0..1))
                        .map(|(fact, _value)| (fact, false)),
                );
                delta.insert(Fact::pack(0, &[owner]), true);
            }
            Event::BecomeFriends { a, b } => {
                delta.insert(Fact::pack(1, &[a, b]), true);
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
                let mut sit = initial_situation.clone();
                for ei in arr {
                    let delta = sit.try_delta(ei.event).unwrap();
                    sit.update(&delta);
                }
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct Filtered<'a>(&'a Situation, bool);
        impl Debug for Filtered<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let iter = self
                    .0
                    .truth
                    .iter()
                    .filter(|&(_fact, &value)| value == self.1)
                    .map(|(&fact, _value)| FactHr(fact));
                f.debug_set().entries(iter).finish()
            }
        }
        f.debug_struct("Situation")
            .field("true", &Filtered(self, true))
            .field("false", &Filtered(self, false))
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
    pub fn write(&mut self, fact_pattern: FactPattern) {
        self.bits &= !fact_pattern.mask;
        self.bits |= fact_pattern.fact.bits;
    }
    pub fn with(mut self, fact_pattern: FactPattern) -> Self {
        self.write(fact_pattern);
        self
    }
    pub const fn read(self, bit_range: Range<u8>) -> u32 {
        (self.bits & bit_mask(range_copy(&bit_range))) >> bit_range.start
    }
}
impl ReplState {
    fn handle_task(&mut self, task: Task) {
        match task {
            Task::AgentHistoryAdd { agent, graph } => self.agent_histories[agent].compose(&graph),
            Task::AgentHistoryPrint { agent } => println!("{:#?}", &self.agent_histories[agent]),
            Task::AgentDestinationsPrint { agent } => {
                let destinations =
                    self.agent_histories[agent].destinations(&self.initial_situation);
                println!("{:#?}", &destinations);
            }
            Task::GlobalHistoryPrint => {
                let global = self
                    .agent_histories
                    .values()
                    .fold(EventGraph::default(), |global, local| global.composed(local));
                println!("{:#?}", &global);
            }
            Task::GlobalDestinationsPrint => {
                let global = self
                    .agent_histories
                    .values()
                    .fold(EventGraph::default(), |global, local| global.composed(local));
                let destinations = global.destinations(&self.initial_situation);
                println!("{:#?}", &destinations);
            }
        }
    }
}

//////////////////////////////////////////////////////

pub fn repl() {
    let initial_history = EventGraph::default();
    let mut repl_state = ReplState {
        initial_situation: Situation::default(),
        agent_histories: enum_map! {
            Agent::Amy => initial_history.clone(),
            Agent::Bob => initial_history.clone(),
            Agent::Dan => initial_history.clone(),
        },
    };
    let [a, b, c] = [
        EventInstance { event: Event::SetOwner { owner: 0 }, index: 0 }, // weh
        EventInstance { event: Event::SetOwner { owner: 1 }, index: 1 }, // weh
        EventInstance { event: Event::SetOwner { owner: 2 }, index: 2 }, // weh
    ];
    repl_state.agent_histories[Agent::Amy].happen.extend([a, c]);
    repl_state.agent_histories[Agent::Amy].before.extend([[a, c]]);

    repl_state.agent_histories[Agent::Bob].happen.extend([a, b]);
    repl_state.agent_histories[Agent::Bob].before.extend([[a, b]]);
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();

    let mut buffer = String::new();
    loop {
        print!("$ ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::io::BufRead::read_line(&mut stdin_lock, &mut buffer).unwrap();
        let task_result = ron::de::from_str::<Task>(&buffer);
        buffer.clear();
        println!("task_result: {task_result:#?}");
        if let Ok(task) = task_result {
            repl_state.handle_task(task);
        }
    }
}
/*
AgentHistoryAdd(agent:Amy,graph:EventGraph(happen:[EventInstance(event:SetOwner(owner:false),index:0),EventInstance(event:SetOwner(owner:false),index:1),EventInstance(event:SetOwner(owner:true),index:2)],before:[(EventInstance(event:SetOwner(owner:false),index:1),EventInstance(event:SetOwner(owner:true),index:2))]))
AgentHistoryPrint(agent:Amy)
AgentDestinationsPrint(agent:Amy)
GlobalHistoryPrint
GlobalDestinationsPrint
    SetOwner { owner: bool },
    BecomeFriends { a: bool, b: bool },
*/
