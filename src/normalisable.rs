use std::cmp;

use crate::event_tree::EventTree;
use crate::id_tree::IdTree;
use crate::stamp::Stamp;


pub trait Normalisable {
    fn norm(self) -> Self;
}

impl Normalisable for IdTree {
    #[allow(non_shorthand_field_patterns)]
    fn norm(self) -> IdTree {
        match self {
            IdTree::Leaf {i: _} => {
                return self;
            }
            IdTree::Node {left, right} => {
                let norm_left = left.norm();
                let norm_right = right.norm();

                if let IdTree::Leaf{i: i1} = norm_left {
                    if let IdTree::Leaf{i: i2} = norm_right {
                        if i1 == i2 {
                            return norm_left;
                        }
                    }
                }

                return IdTree::node(Box::new(norm_left), Box::new(norm_right));
            }
        };
    }
}

impl Normalisable for EventTree {
    fn norm(self) -> EventTree {
        match self {
            EventTree::Leaf {n: _} => {
                return self;
            },
            EventTree::Node {n, left, right} => {
                let norm_left = left.norm();
                let norm_right = right.norm();

                if let EventTree::Leaf{n: m1} = norm_left {
                    if let EventTree::Leaf{n: m2} = norm_right {
                        if m1 == m2 {
                            return EventTree::leaf(n + m1);
                        }
                    }
                }

                // normalised trees have min == n
                let min_left = norm_left.n();
                let min_right = norm_right.n();

                let m = cmp::min(min_left, min_right);

                return EventTree::node(n + m, Box::new(norm_left.sink(m)), Box::new(norm_right.sink(m)));
            }
        }
    }
}

impl Normalisable for Stamp {
    fn norm(self) -> Stamp {
        Stamp::new(self.i.norm(), self.e.norm())
    }
}
