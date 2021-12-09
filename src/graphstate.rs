use core::ops::Range;

type Vert = u32;
type Label = u32;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct LabelledHalfEdge {
    vert: Vert,
    label: Label,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    from_n_label: LabelledHalfEdge,
    to: Vert,
}

#[derive(Debug)]
struct Graph {
    sorted_edges: Vec<Edge>, // sorted on (from, label)
}
impl Graph {
    fn from_edges(mut edges: Vec<Edge>) -> Self {
        edges.sort_by_key(|edge| edge.from_n_label);
        Self { sorted_edges: edges }
    }
    fn steps_to(&self, from_n_label: &LabelledHalfEdge) -> Option<Vert> {
        let i =
            self.sorted_edges.binary_search_by_key(from_n_label, |edge| edge.from_n_label).ok()?;
        Some(self.sorted_edges[i].to)
    }
    fn has_outgoing(&self, vert: Vert) -> bool {
        self.sorted_edges.binary_search_by_key(&vert, |edge| edge.from_n_label.vert).is_ok()
    }
    fn deterministic_step(&self, vert: Vert) -> Option<LabelledHalfEdge> {
        let s = &self.sorted_edges;
        let i = s.binary_search_by_key(&vert, |edge| edge.from_n_label.vert).ok()?;
        // check that this vert has no other outgoing: constant probe
        let none_smaller = || i == 0 || s[i - 1].from_n_label.vert != vert;
        let none_larger = || i == s.len() - 1 || s[i + 1].from_n_label.vert != vert;
        if none_smaller() && none_larger() {
            let edge = &s[i];
            Some(LabelledHalfEdge { vert: edge.to, label: edge.from_n_label.label })
        } else {
            None
        }
    }
    fn at_vert(&self, vert: Vert) -> &[Edge] {
        // taken from https://www.geeksforgeeks.org/find-first-and-last-positions-of-an-element-in-a-sorted-array/
        let s = &self.sorted_edges;
        let at = |idx: usize| s[idx].from_n_label.vert;
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
        Edge { from_n_label: LabelledHalfEdge { vert: 0, label: 420 }, to: 1 },
        Edge { from_n_label: LabelledHalfEdge { vert: 0, label: 69 }, to: 2 },
        Edge { from_n_label: LabelledHalfEdge { vert: 0, label: 80085 }, to: 0 },
        Edge { from_n_label: LabelledHalfEdge { vert: 1, label: 111 }, to: 0 },
        Edge { from_n_label: LabelledHalfEdge { vert: 2, label: 123 }, to: 1 },
    ]);
    println!("{:#?}", g);

    let mut at = 0;
    for label in [80085, 80085, 69, 123, 111, 80085] {
        at = g.steps_to(&LabelledHalfEdge { vert: at, label }).unwrap();
        println!("at = {}", at);
    }
    println!("{:?}", g.at_vert(at).len());
}
