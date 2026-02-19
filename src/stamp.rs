use std::cmp::Ordering;
use std::{borrow::Cow, cmp};

use crate::cost::Cost;
use crate::event_tree::{EventTree, Max, Min};
use crate::id_tree::IdTree;
use crate::normalisable::Normalisable;


#[derive(Debug, Clone, PartialEq)]
pub struct Stamp {
    pub(crate) i: IdTree,
    pub(crate) e: EventTree
}

impl Stamp {
    pub fn seed() -> Stamp {
        Stamp::new(IdTree::one(), EventTree::zero())
    }

    pub fn new(i: IdTree, e: EventTree) -> Stamp {
        Stamp {
            i: i,
            e: e
        }
    }

    pub fn fill<'a>(&'a self) -> Cow<'a, EventTree> {
        if self.i == IdTree::zero() {
            Cow::Borrowed(&self.e)
        } else if self.i == IdTree::one() {
            Cow::Owned(EventTree::leaf(self.e.max()))
        } else if let EventTree::Leaf {..} = self.e {
            Cow::Borrowed(&self.e)
        } else {
            if let IdTree::Node {left: ref i_left, right: ref i_right} = self.i {
                if let EventTree::Node {n, left: ref e_left, right: ref e_right} = self.e {
                    if i_left.as_ref() == &IdTree::one() {
                        let eprime_right = Stamp::new(i_right.as_ref().clone(), e_right.as_ref().clone()).fill().into_owned();
                        let new_left = EventTree::leaf(cmp::max(e_left.max(), eprime_right.min()));
                        Cow::Owned(EventTree::node(n, Box::new(new_left), Box::new(eprime_right)).norm())
                    } else if i_right.as_ref() == &IdTree::one() {
                        let eprime_left = Stamp::new(i_left.as_ref().clone(), e_left.as_ref().clone()).fill().into_owned();
                        let new_right = EventTree::leaf(cmp::max(e_right.max(), eprime_left.min()));
                        Cow::Owned(EventTree::node(n, Box::new(eprime_left), Box::new(new_right)).norm())
                    } else {
                        let new_left = Stamp::new(i_left.as_ref().clone(), e_left.as_ref().clone()).fill().into_owned();
                        let new_right = Stamp::new(i_right.as_ref().clone(), e_right.as_ref().clone()).fill().into_owned();
                        Cow::Owned(EventTree::node(n, Box::new(new_left), Box::new(new_right)).norm())
                    }
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        }
    }

    // returns event tree and cost
    pub fn grow(&self) -> (EventTree, Cost) {
        match self.e {
            EventTree::Leaf {n} => {
                if self.i == IdTree::one() {
                    (EventTree::leaf(n + 1), Cost::zero())
                } else {
                    let new_e = EventTree::node(n, Box::new(EventTree::zero()), Box::new(EventTree::zero()));
                    let (eprime, c) = Stamp::new(self.i.clone(), new_e).grow();
                    (eprime, c.shift())
                }
            },
            EventTree::Node {n, left: ref e_left, right: ref e_right} => {
                if let IdTree::Node {left: ref i_left, right: ref i_right} = self.i {
                    if **i_left == IdTree::zero() {
                        let (eprime_right, c_right) = Stamp::new(i_right.as_ref().clone(), e_right.as_ref().clone()).grow();
                        (EventTree::node(n, e_left.clone(), Box::new(eprime_right)), c_right + 1)
                    } else if **i_right == IdTree::zero() {
                        let (eprime_left, c_left) = Stamp::new(*i_left.clone(), *e_left.clone()).grow();
                        (EventTree::node(n, Box::new(eprime_left), e_right.clone()), c_left + 1)
                    } else {
                        let (eprime_right, c_right) = Stamp::new(*i_right.clone(), *e_right.clone()).grow();
                        let (eprime_left, c_left) = Stamp::new(*i_left.clone(), *e_left.clone()).grow();
                        if c_left < c_right {
                            (EventTree::node(n, Box::new(eprime_left), e_right.clone()), c_left + 1)
                        } else {
                            (EventTree::node(n, e_left.clone(), Box::new(eprime_right)), c_right + 1)
                        }
                    }
                } else {
                    // corrupted tree?
                    unreachable!()
                }
            }
        }
    }
}

impl PartialOrd for Stamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.e.partial_cmp(&other.e)
    }
}
