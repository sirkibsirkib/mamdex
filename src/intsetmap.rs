use std::collections::HashSet;
use std::sync::Arc;

#[derive(Default, Debug, Eq, PartialEq)]
struct IntSet {
    varints: HashSet<Box<[u8]>>,
}

#[derive(Debug, Clone)]
enum IntSetMap {
    Identity,
    Constant(Arc<IntSet>),
    UnionWith(Arc<Self>),
    IntersectWith(Arc<Self>),
    IfEqThenElse { i: Arc<Self>, t: Arc<Self>, e: Arc<Self> },
    EnumerateThenUnion(Arc<Self>),
}

impl IntSetMap {
    fn apply_to(&self, arg: &Arc<IntSet>) -> Arc<IntSet> {
        use IntSetMap::*;
        match self {
            Identity => arg.clone(),
            Constant(x) => x.clone(),
            UnionWith(x) => Arc::new(IntSet {
                varints: arg.varints.union(&x.apply_to(arg).varints).cloned().collect(),
            }),
            IntersectWith(x) => Arc::new(IntSet {
                varints: arg.varints.intersection(&x.apply_to(arg).varints).cloned().collect(),
            }),
            IfEqThenElse { i, t, e } => {
                let i = &i.apply_to(arg);
                match i == arg {
                    true => t,
                    false => e,
                }
                .apply_to(arg)
            }
            EnumerateThenUnion(elements) => {
                let mut result = IntSet::default();
                for varint in elements.apply_to(arg).varints.iter() {
                    let singleton = IntSet { varints: std::iter::once(varint.clone()).collect() };
                    let out = 
                    result = result.union()
                }
            }
        }
    }
}
