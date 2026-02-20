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
    pub fn zero() -> Self {
        Self::leaf(0)
    }

    pub fn leaf(n: u32) -> Self {
        Self::Leaf {
            n: n
        }
    }

    pub fn node(n: u32, left: Box<Self>, right: Box<Self>) -> Self {
        Self::Node {
            n: n,
            left: left,
            right: right
        }
    }

    pub fn n(&self) -> u32 {
        match self {
            Self::Leaf { n } => *n,
            Self::Node { n, .. } => *n
        }
    }

    pub fn lift(&mut self, m: u32) -> &mut Self {
        match self {
            Self::Leaf { n } => *n += m,
            Self::Node { n, .. } => *n += m,
        };

        self
    }

    pub fn sink(&mut self, m: u32) -> &mut Self {
        match self {
            Self::Leaf { n } => *n -= m,
            Self::Node { n, .. } => *n -= m,
        };

        self
    }

    pub fn join(&self, other: &Self) -> Self {
        use self::EventTree::*;
        match (self, other) {
            (Leaf { n: n1 }, Leaf { n: n2 }) => Self::leaf(max(*n1, *n2)),
            (Leaf { n: n1 }, right) => {
                Self::node(*n1, Box::new(Self::zero()), Box::new(Self::zero())).join(right)
            },
            (left, Leaf { n: n2 }) => {
                left.join(&Self::node(*n2, Box::new(Self::zero()), Box::new(Self::zero())))
            },
            (Node { n: n1, .. }, Node { n: n2, ..}) if *n1 > *n2 => other.join(self),
            (Node { n: n1, left: left1, right: right1 }, Node { n: n2, left: left2, right: right2 }) => {
                let diff = n2 - n1;
                let new_left = left1.join(left2.clone().lift(diff));
                let new_right = right1.join(right2.clone().lift(diff));

                Self::node(*n1, Box::new(new_left), Box::new(new_right)).norm()
            },
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

        use self::EventTree::*;
        match (first, second) {
            (Leaf { n: n1 }, Leaf { n: n2 }) => n1.partial_cmp(&n2),
            (Leaf { n: n1 }, Node { n: n2, mut left, mut right }) => {
                vec![
                    n1.partial_cmp(&n2),
                    Leaf { n: n1 }.partial_cmp(&left.lift(n2)),
                    Leaf { n: n1 }.partial_cmp(&right.lift(n2)),
                ].into_iter().reduce(two_way_cmp).flatten()
            },
            (Node { n: n1, mut left, mut right }, Leaf { n: n2 }) => {
                vec![
                    n1.partial_cmp(&n2),
                    (*left.lift(n1)).partial_cmp(&Leaf { n: n2 }),
                    (*right.lift(n1)).partial_cmp(&Leaf { n: n2 }),
                ].into_iter().reduce(two_way_cmp).flatten()
            },
            (Node { n: n1, left: mut left1, right: mut right1 }, Node { n: n2, left: mut left2, right: mut right2 }) => {
                vec![
                    n1.partial_cmp(&n2),
                    (*left1.lift(n1)).partial_cmp(left2.lift(n2)),
                    (*right1.lift(n1)).partial_cmp(right2.lift(n2)),
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
        match self {
            Self::Leaf{n} => *n,
            Self::Node{n, left, right} => n + min(left.min(), right.min())
        }
    }
}


pub trait Max<T> {
    fn max(&self) -> T;
}

impl Max<u32> for EventTree {
    fn max(&self) -> u32 {
        match self {
            Self::Leaf{n} => *n,
            Self::Node{n, left, right} => n + max(left.max(), right.max())
        }
    }
}
