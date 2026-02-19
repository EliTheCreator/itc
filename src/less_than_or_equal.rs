use crate::{event_tree::EventTree, stamp::Stamp};


pub trait LessThanOrEqual {
    fn leq(&self, other: &Self) -> bool;
}

impl LessThanOrEqual for Stamp {
    fn leq(&self, other: &Stamp) -> bool {
        self.e.leq(&other.e)
    }
}

impl LessThanOrEqual for EventTree {
    #[allow(non_shorthand_field_patterns)]
    fn leq(&self, other: &EventTree) -> bool {
        match *self {
            EventTree::Leaf {n: n1} => {
                match *other {
                    EventTree::Leaf {n: n2} => {
                        n1 <= n2
                    },
                    EventTree::Node {n: n2, ..} => {
                        n1 <= n2
                    }
                }
            },
            EventTree::Node {n: n1, left: ref left1, right: ref right1} => {
                match *other {
                    EventTree::Leaf {n: n2} => {
                        (n1 <= n2)
                        && left1.clone().lift(n1).leq(&EventTree::leaf(n2))
                        && right1.clone().lift(n1).leq(&EventTree::leaf(n2))
                    },
                    EventTree::Node {n: n2, left: ref left2, right: ref right2} => {
                        (n1 <= n2)
                        && left1.clone().lift(n1).leq(&left2.clone().lift(n2))
                        && right1.clone().lift(n1).leq(&right2.clone().lift(n2))
                    }
                }
            }
        }
    }
}
