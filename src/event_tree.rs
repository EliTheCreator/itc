use std::cmp::{Ordering, max, min};

use crate::normalisable::Normalisable;


#[derive(Debug, Clone, PartialEq)]
pub enum EventTree {
    Leaf {
        n: u32
    },
    Node {
        n: u32,
        left: Box<EventTree>,
        right: Box<EventTree>
    }
}

impl EventTree {
    pub fn zero() -> EventTree {
        EventTree::leaf(0)
    }

    pub fn leaf(n: u32) -> EventTree {
        EventTree::Leaf {
            n: n
        }
    }

    pub fn node(n: u32, left: Box<EventTree>, right: Box<EventTree>) -> EventTree {
        EventTree::Node {
            n: n,
            left: left,
            right: right
        }
    }

    pub fn n(&self) -> u32 {
        match self {
            &EventTree::Leaf { n } => n,
            &EventTree::Node { n, .. } => n
        }
    }

    pub fn lift(self, m: u32) -> EventTree {
        match self {
            EventTree::Leaf { n } => EventTree::leaf(n + m),
            EventTree::Node { n, left, right } => EventTree::node(n + m, left, right)
        }
    }

    pub fn sink(self, m: u32) -> EventTree {
        match self {
            EventTree::Leaf { n } => EventTree::leaf(n - m),
            EventTree::Node { n, left, right } => EventTree::node(n - m, left, right)
        }
    }

    pub fn join(&self, other: &EventTree) -> EventTree {
        match *self {
            EventTree::Leaf {n: n1} => {
                match *other {
                    EventTree::Leaf {n: n2} => {
                        EventTree::leaf(max(n1, n2))
                    },
                    EventTree::Node {..} => {
                        let new_left = EventTree::node(n1, Box::new(EventTree::zero()), Box::new(EventTree::zero()));
                        new_left.join(other)
                    }
                }
            },
            EventTree::Node {n: n1, left: ref left1, right: ref right1} => {
                match *other {
                    EventTree::Leaf {n: n2} => {
                        let new_right = EventTree::node(n2, Box::new(EventTree::zero()), Box::new(EventTree::zero()));
                        self.join(&new_right)
                    },
                    EventTree::Node {n: n2, left: ref left2, right: ref right2} => {
                        if n1 > n2 {
                            other.join(self)
                        } else {
                            let new_left = left1.join(&left2.clone().lift(n2 - n1));
                            let new_right = right1.join(&right2.clone().lift(n2 - n1));
                            EventTree::node(n1, Box::new(new_left), Box::new(new_right)).norm()
                        }
                    }
                }
            }
        }
    }
}

impl PartialOrd for EventTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let first = self.clone().norm();
        let second = other.clone().norm();

        let two_way_cmp = {
            | l: Option<Ordering>, r: Option<Ordering> | -> Option<Ordering> {
                use std::cmp::Ordering::*;
                match (l, r) {
                    (_, None) | (None, _) => None,
                    (Some(Less), Some(Greater)) => None,
                    (Some(Greater), Some(Less)) => None,
                    (Some(Less), Some(Equal)) => Some(Less),
                    (Some(Equal), Some(Less)) => Some(Less),
                    (Some(Equal), Some(Greater)) => Some(Greater),
                    (Some(Greater), Some(Equal)) => Some(Greater),
                    (Some(Less), Some(Less)) => Some(Less),
                    (Some(Equal), Some(Equal)) => Some(Equal),
                    (Some(Greater), Some(Greater)) => Some(Greater),
                }
            }
        };

        match (first, second) {
            (EventTree::Leaf { n: n1 }, EventTree::Leaf { n: n2 }) => n1.partial_cmp(&n2),
            (EventTree::Leaf { n: n1 }, EventTree::Node { n: n2, left, right }) => {
                vec![
                    n1.partial_cmp(&n2),
                    EventTree::Leaf { n: n1 }.partial_cmp(&left.lift(n2)),
                    EventTree::Leaf { n: n1 }.partial_cmp(&right.lift(n2)),
                ].into_iter().reduce(two_way_cmp).flatten()
            },
            (EventTree::Node { n: n1, left, right }, EventTree::Leaf { n: n2 }) => {
                vec![
                    n1.partial_cmp(&n2),
                    left.lift(n1).partial_cmp(&EventTree::Leaf { n: n2 }),
                    right.lift(n1).partial_cmp(&EventTree::Leaf { n: n2 }),
                ].into_iter().reduce(two_way_cmp).flatten()
            },
            (EventTree::Node { n: n1, left: left1, right: right1 }, EventTree::Node { n: n2, left: left2, right: right2 }) => {
                vec![
                    n1.partial_cmp(&n2),
                    left1.lift(n1).partial_cmp(&left2.lift(n2)),
                    right1.lift(n1).partial_cmp(&right2.lift(n2)),
                ].into_iter().reduce(two_way_cmp).flatten()
            },
        }
    }
}


pub trait Min<T> {
    fn min(&self) -> T;
}

impl Min<u32> for EventTree {
    fn min(&self) -> u32 {
        match *self {
            EventTree::Leaf{n} => n,
            EventTree::Node{n, ref left, ref right} => n + min(left.min(), right.min())
        }
    }
}


pub trait Max<T> {
    fn max(&self) -> T;
}

impl Max<u32> for EventTree {
    fn max(&self) -> u32 {
        match *self {
            EventTree::Leaf{n} => n,
            EventTree::Node{n, ref left, ref right} => n + max(left.max(), right.max())
        }
    }
}