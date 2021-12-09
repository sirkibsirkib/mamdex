use core::ops::Range;

type Vert = u32;
type Value = u32;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct VertValue {
    vert: Vert,
    value: Value,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    from_value: VertValue,
    to: Vert,
}

#[derive(Debug)]
struct Graph {
    sorted_edges: Vec<Edge>, // sorted on (from, value)
}
impl Graph {
    fn from_edges(mut edges: Vec<Edge>) -> Self {
        edges.sort_by_key(|edge| edge.from_value);
        Self { sorted_edges: edges }
    }
    fn steps_to(&self, from_value: &VertValue) -> Option<Vert> {
        let i = self.sorted_edges.binary_search_by_key(from_value, |edge| edge.from_value).ok()?;
        Some(self.sorted_edges[i].to)
    }
    fn has_outgoing(&self, vert: Vert) -> bool {
        self.sorted_edges.binary_search_by_key(&vert, |edge| edge.from_value.vert).is_ok()
    }
    fn at_vert(&self, vert: Vert) -> &[Edge] {
        let s = &self.sorted_edges;
        let at = |idx: usize| s[idx].from_value.vert;
        let middle = |range: Range<usize>| range.start + range.len() / 2;
        let first_at = {
            let mut range = 0..s.len();
            loop {
                if range.is_empty() {
                    return &[];
                }
                let mid = middle(range.clone());
                if (mid == 0 || vert > at(mid - 1)) && at(mid) == vert {
                    break mid;
                }
                if vert > at(mid) {
                    range.start = mid + 1;
                } else {
                    range.end = mid - 1;
                }
            }
        };

        let last = {
            let mut range = 0..s.len();
            loop {
                let mid = middle(range.clone());
                if (mid == s.len() - 1 || vert < at(mid + 1)) && at(mid) == vert {
                    break mid;
                }
                if vert < at(mid) {
                    range.end = mid - 1;
                } else {
                    range.start = mid + 1;
                }
            }
        };
        &s[first_at..=last]
    }
}

#[test]
pub fn q() {
    let g = Graph::from_edges(vec![
        Edge { from_value: VertValue { vert: 0, value: 420 }, to: 1 },
        Edge { from_value: VertValue { vert: 0, value: 69 }, to: 2 },
        Edge { from_value: VertValue { vert: 0, value: 80085 }, to: 0 },
        Edge { from_value: VertValue { vert: 1, value: 111 }, to: 0 },
        Edge { from_value: VertValue { vert: 2, value: 123 }, to: 1 },
    ]);
    println!("{:#?}", g);

    let mut at = 0;
    for value in [80085, 80085, 69, 123, 111, 80085] {
        at = g.steps_to(&VertValue { vert: at, value }).unwrap();
        println!("at = {}", at);
    }
    println!("{:?}", g.at_vert(at).len());
}
