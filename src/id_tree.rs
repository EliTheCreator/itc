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
    fn split(&self) -> Self;
}

impl Split for IdTree {
    fn split(&self) -> IdTree {
        match *self {
            IdTree::Leaf {i} => {
                if !i {
                    IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::zero()))
                } else {
                    let new_left = Box::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())));
                    let new_right = Box::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one())));
                    IdTree::node(new_left, new_right)
                }
            },
            IdTree::Node {ref left, ref right} => {
                if *left.as_ref() == IdTree::zero() {
                    // split always returns a Node, not a Leaf
                    if let IdTree::Node{left: i1, right: i2} = right.split() {
                        let new_left = Box::new(IdTree::node(Box::new(IdTree::zero()), i1));
                        let new_right = Box::new(IdTree::node(Box::new(IdTree::zero()), i2));
                        IdTree::node(new_left, new_right)
                    } else {
                        unreachable!()
                    }
                } else if *right.as_ref() == IdTree::zero() {
                    if let IdTree::Node{left: i1, right: i2} = left.split() {
                        let new_left = Box::new(IdTree::node(i1, Box::new(IdTree::zero())));
                        let new_right = Box::new(IdTree::node(i2, Box::new(IdTree::zero())));
                        IdTree::node(new_left, new_right)
                    } else {
                        unreachable!()
                    }
                } else {
                    let new_left = Box::new(IdTree::node(left.clone(), Box::new(IdTree::zero())));
                    let new_right = Box::new(IdTree::node(Box::new(IdTree::zero()), right.clone()));
                    IdTree::node(new_left, new_right)
                }
            }
        }
    }
}


pub trait Sum {
    fn sum(&self, other: &Self) -> Self;
}

impl Sum for IdTree {
    #[allow(non_shorthand_field_patterns)]
    fn sum(&self, other: &IdTree) -> IdTree {
        if *self == IdTree::zero() {
            return other.clone();
        } else if *other == IdTree::zero() {
            return self.clone();
        }

        if let IdTree::Node {left: ref left1, right: ref right1} = *self {
            if let IdTree::Node {left: ref left2, right: ref right2} = *other {
                let new_left = Box::new(left1.sum(left2));
                let new_right = Box::new(right1.sum(right2));
                return IdTree::node(new_left, new_right).norm();
            } else {
                // corrupted tree?
                unreachable!();
            }
        } else {
            // corrupted tree?
            unreachable!();
        }
    }
}
