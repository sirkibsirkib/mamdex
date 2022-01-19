use maplit::{btreemap as btm, hashmap as hm, hashset as hs};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Event(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Fluent(u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Step {
    vert: Event,
    preds: BTreeSet<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Graph {
    vert_to_preds: HashMap<Event, BTreeSet<Event>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Situation {
    valuations: BTreeMap<Fluent, bool>,
}

struct EventFn(fn(HashSet<Situation>) -> HashSet<Situation>);

struct Running<'a> {
    g: &'a Graph,
    input: HashSet<Situation>,
    events_out: HashMap<BTreeSet<Event>, HashSet<Situation>>,
    event_fns: &'a EventFns,
}

struct EventFns(HashMap<Event, EventFn>);

trait SituationTransformer {
    fn transform(&self, event_fns: &EventFns, input: HashSet<Situation>) -> HashSet<Situation>;
}

/////////////////////////

impl SituationTransformer for Event {
    fn transform(&self, event_fns: &EventFns, input: HashSet<Situation>) -> HashSet<Situation> {
        (event_fns.0.get(self).unwrap().0)(input)
    }
}
impl SituationTransformer for (Graph, BTreeSet<Event>) {
    fn transform(&self, event_fns: &EventFns, input: HashSet<Situation>) -> HashSet<Situation> {
        todo!()
    }
}

// fn order_fold([mut a, b]: [Situation; 2]) -> Situation {
//     for (fluent, value) in b.valuations {
//         a.valuations.insert(fluent, value);
//     }
//     a
// }
// fn situation_pair_fold([a, b]: [Situation; 2]) -> [Situation; 2] {
//     [order_fold([a.clone(), b.clone()]), order_fold([b, a])]
// }
// fn situation_set_fold(a: HashSet<Situation>, b: HashSet<Situation>) -> HashSet<Situation> {
//     let mut out = HashSet::<Situation>::default();
//     for a in &a {
//         for b in &b {
//             let [x, y] = situation_pair_fold([a.clone(), b.clone()]);
//             out.insert(x);
//             out.insert(y);
//         }
//     }
//     out
// }
// impl Running<'_> {
//     fn out_for_terminals(&mut self) -> &HashSet<Situation> {
//         self.out_for(&self.g.terminals())
//     }
//     fn out_for(&mut self, events: &BTreeSet<Event>) -> &HashSet<Situation> {
//         let mut event_func = |event| {
//             let pred_outs = if let Some(preds) = self.g.vert_to_preds.get(event) {
//                 self.out_for(preds).clone()
//             } else {
//                 self.input.clone()
//             };
//             let event_fn = self.event_fns.0.get(event).unwrap();
//             (event_fn.0)(pred_outs)
//         };
//         if !self.events_out.contains_key(events) {
//             // compute it
//             let value = match events.len() {
//                 0 => self.input.clone(),
//                 1 => {
//                     let event = events.iter().next().unwrap();
//                     event_func(event)
//                 }
//                 _ => {
//                     let init = hs! {Situation {valuations: Default::default()}};
//                     events
//                         .iter()
//                         .map(|&event| {
//                             let singleton: BTreeSet<Event> = Some(event).into_iter().collect();
//                             self.out_for(&singleton).clone()
//                         })
//                         .fold(init, situation_set_fold)
//                     //
//                 }
//             };
//             self.events_out.insert(events.clone(), value);
//         }
//         self.events_out.get(events).unwrap()
//     }
// }

// impl Graph {
//     fn combined(mut self, other: Self) -> Self {
//         for (vert, preds) in other.vert_to_preds.into_iter() {
//             use std::collections::hash_map::Entry;
//             match self.vert_to_preds.entry(vert) {
//                 Entry::Occupied(mut o) => o.get_mut().extend(preds),
//                 Entry::Vacant(v) => drop(v.insert(preds)),
//             }
//         }
//         self
//     }
//     fn initials(&self) -> impl Iterator<Item = Event> + '_ {
//         self.vert_to_preds
//             .iter()
//             .filter_map(|(&vert, preds)| if preds.is_empty() { Some(vert) } else { None })
//     }
//     fn terminals(&self) -> BTreeSet<Event> {
//         let preds: HashSet<Event> = self.vert_to_preds.values().flat_map(|x| x).cloned().collect();
//         self.vert_to_preds.keys().cloned().filter(|vert| !preds.contains(vert)).collect()
//     }
// }

// #[test]
// fn folding() {
//     let a = hs! {
//         Situation { valuations: btm! { Fluent(0) => true, Fluent(1) => true } },
//     };
//     let b = hs! {
//         Situation { valuations: btm! { Fluent(0) => false } },
//         Situation { valuations: btm! { Fluent(0) => true } },
//         Situation { valuations: btm! { Fluent(0) => false } },
//     };
//     println!("{:#?}", situation_set_fold(a, b));
// }

// #[test]
// fn yarp() {
//     let er = EventFns(hm! {
//         Event(0) => EventFn(|s| s)
//     });
// }
