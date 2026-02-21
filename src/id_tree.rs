use crate::normalisable::Normalisable;


#[derive(Debug, Clone, PartialEq)]
pub enum IdTree {
    Leaf {
        i: bool
    },
    Node {
        left: Box<IdTree>,
        right: Box<IdTree>
    }
}

impl IdTree {
    pub fn leaf(i: bool) -> IdTree {
        IdTree::Leaf {
            i: i
        }
    }

    pub fn zero() -> IdTree {
        IdTree::Leaf {
            i: false
        }
    }

    pub fn one() -> IdTree {
        IdTree::Leaf {
            i: true
        }
    }

    pub fn node(left: Box<IdTree>, right: Box<IdTree>) -> IdTree {
        IdTree::Node {
            left: left,
            right: right
        }
    }
}


pub trait Split {
    fn split(&self) -> (Self, Self) where Self: Sized;
}

impl Split for IdTree {
    fn split(&self) -> (Self, Self) {
        use self::IdTree::*;
        match self {
            Leaf { i: false } =>  (Self::zero(), Self::zero()),
            Leaf { i: true } => {
                let left = Self::node(Box::new(Self::one()), Box::new(Self::zero()));
                let right = Self::node(Box::new(Self::zero()), Box::new(Self::one()));
                (left, right)
            },
            // TODO: rewrite this as multiple cases instead of a second match
            //       statement if/once the box syntax is stabilised
            Node { left, right } => {
                match (left.as_ref(), right.as_ref()) {
                    (Leaf { i: false }, right) => {
                        let (split1, split2) = right.split();

                        let new_left = Self::node(Box::new(Self::zero()), Box::new(split1));
                        let new_right = Self::node(Box::new(Self::zero()), Box::new(split2));
                        (new_left, new_right)
                    },
                    (left, Leaf { i: false }) => {
                        let (split1, split2) = left.split();

                        let new_left = Self::node(Box::new(split1), Box::new(Self::zero()));
                        let new_right = Self::node(Box::new(split2), Box::new(Self::zero()));
                        (new_left, new_right)
                    },
                    (left, right ) => {
                        let new_left = Self::node(Box::new(left.clone()), Box::new(Self::zero()));
                        let new_right = Self::node(Box::new(Self::zero()), Box::new(right.clone()));
                        (new_left, new_right)
                    },
                }
            },
        }
    }
}


pub trait Sum {
    fn sum(&self, other: &Self) -> Self;
}

impl Sum for IdTree {
    fn sum(&self, other: &Self) -> Self {
        use self::IdTree::*;
        match (self, other) {
            (_, Leaf { i: false }) => self.clone(),
            (Leaf { i: false }, _) => other.clone(),
            (Node { left: left1, right: right1 }, Node { left: left2, right: right2 }) => {
                let new_left = Box::new(left1.sum(left2));
                let new_right = Box::new(right1.sum(right2));
                Self::node(new_left, new_right).norm()
            },
            _ => unreachable!()
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{IdTree, Split};

    #[test]
    fn split_test() {
        assert_eq!(IdTree::one().split(), (IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one()))));
    }
}
