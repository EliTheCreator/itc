use std::cmp::min;

use crate::event_tree::EventTree;
use crate::id_tree::IdTree;
use crate::stamp::Stamp;


pub trait Normalisable {
    fn norm(self) -> Self;
}

impl Normalisable for IdTree {
    #[allow(non_shorthand_field_patterns)]
    fn norm(self) -> Self {
        use self::IdTree::*;
        match self {
            Leaf {i: _} => self,
            Node {left, right} => {
                let norm_left = left.norm();
                let norm_right = right.norm();

                match (&norm_left, &norm_right) {
                    (Leaf { i: i1 }, Leaf { i: i2 }) if i1==i2 => norm_left,
                    _ => Self::node(Box::new(norm_left), Box::new(norm_right)),
                }
            }
        }
    }
}

impl Normalisable for EventTree {
    fn norm(self) -> EventTree {
        use self::EventTree::*;
        match self {
            Leaf {n: _} => self,
            Node {n, left, right} => {
                let mut norm_left = left.norm();
                let mut norm_right = right.norm();

                match (&norm_left, &norm_right) {
                    (Leaf { n: m1 }, Leaf { n: m2 }) if m1==m2 => Self::leaf(n+m1),
                    _ => {
                        let m = min(norm_left.n(), norm_right.n());
                        norm_left.sink(m);
                        norm_right.sink(m);
                        Self::node(n + m, Box::new(norm_left), Box::new(norm_right))
                    }
                }
            }
        }
    }
}

impl Normalisable for Stamp {
    fn norm(self) -> Self {
        Self::new(self.i.norm(), self.e.norm())
    }
}
