use core::ops::Not;
use std::collections::HashMap;

use core::ops::BitAnd;
use maplit::hashmap as hm;

struct MaskedRule {
    if_all: usize,
    then_all: usize,
    then_none: usize,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Var(u16);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Val(u16);

struct Action {
    pre_cond: HashMap<Var, Val>,
    post_cond: HashMap<Var, Val>,
}

#[derive(Debug)]
struct State {
    store: HashMap<Var, Val>,
}

struct Spec {
    actions: Vec<Action>,
    masked_rules: Vec<MaskedRule>,
}

trait Bmap: Into<bool> + Sized {
    fn bmap<T>(self, t: T) -> Option<T> {
        if self.into() {
            Some(t)
        } else {
            None
        }
    }
}
impl Bmap for bool {}
trait BitMask: Copy + PartialEq + Not<Output = Self> + BitAnd<Output = Self> {
    const NULL: Self;
    fn without(self, other: Self) -> Self {
        self & !other
    }
    fn is_superset_of(self, other: Self) -> bool {
        other.without(self) == Self::NULL
    }
    fn overlaps(self, other: Self) -> bool {
        self & other != Self::NULL
    }
    fn singleton_nth_lsb(n: usize) -> Self;
    fn nth_lsb(self, n: usize) -> bool {
        self.overlaps(Self::singleton_nth_lsb(n))
    }
}
impl BitMask for usize {
    const NULL: Self = 0;
    fn singleton_nth_lsb(n: usize) -> Self {
        1 << n
    }
}

fn new_state(old_state: &State, action_mask: usize, actions: &[Action]) -> Option<State> {
    let mut new_state = State { store: hm! {} };
    let action_iter = actions
        .iter()
        .enumerate()
        .filter_map(|(i, a)| action_mask.nth_lsb(i).bmap(a));
    for action in action_iter {
        for (k, v) in action.pre_cond.iter() {
            match old_state.store.get(&k) {
                Some(v2) if v2 == v => {}
                _ => return None,
            }
        }
        for (k, v) in action.post_cond.iter() {
            match new_state.store.insert(*k, *v) {
                Some(v2) if v2 != *v => return None,
                _ => {}
            }
        }
    }
    println!("NEW ONLY {:?}", new_state);
    for (k, v) in old_state.store.iter() {
        new_state.store.entry(*k).or_insert(*v);
    }
    Some(new_state)
}

fn zop(spec: &Spec, state: &State) {
    assert!(!spec.actions.is_empty());
    let all = (1 << spec.actions.len()) - 1;
    println!("{:>10b} <--ALL", all);
    let mut action_mask = 0usize;
    loop {
        for rule in spec.masked_rules.iter() {
            if action_mask.is_superset_of(rule.if_all) {
                action_mask |= rule.then_all;
            }
            if action_mask.overlaps(rule.then_none) {
                if action_mask >= all {
                    break;
                }
                action_mask += 1;
            }
        }
        println!("{:>10b} LOGICALLY OK", action_mask);
        if let Some(new_state) = new_state(state, action_mask, &spec.actions) {
            println!("ACTIONS OK; NEW STATE: {:?}", new_state);
        }
        if action_mask >= all {
            break;
        }
        action_mask += 1;
    }
}

fn main() {
    let spec = Spec {
        actions: vec![
            Action {
                pre_cond: hm! { Var(0) => Val(0) },
                post_cond: hm! { Var(0) => Val(1)},
            },
            Action {
                pre_cond: hm! {},
                post_cond: hm! { Var(1) => Val(2)},
            },
        ],
        masked_rules: vec![
            MaskedRule {
                if_all: 0b00001,
                then_all: 0b00011,
                then_none: 0b0000,
            }, //yass
            MaskedRule {
                if_all: 0b00000,
                then_all: 0b00000,
                then_none: 0b00000,
            }, //yass
        ],
    };
    let state = State {
        store: hm! { Var(0) => Val(0)},
    };
    zop(&spec, &state);
}
