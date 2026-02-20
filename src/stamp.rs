use std::cmp::{Ordering, max};

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

    pub fn fill(&self) -> EventTree {
        match (&self.i, &self.e) {
            (IdTree::Leaf { i: false }, _) => self.e.clone(),
            (IdTree::Leaf { i: true }, _) => EventTree::leaf(self.e.max()),
            (_, EventTree::Leaf { .. }) => self.e.clone(),
            // TODO: rewrite this as multiple cases instead of a second match
            //       statement if/once the box syntax is stabilised
            (IdTree::Node { left: i_l, right: i_r }, EventTree::Node { n, left: e_l, right: e_r }) => {
                match (i_l.as_ref(), i_r.as_ref()) {
                    (IdTree::Leaf { i: true }, _) => {
                        let eprime_r = Stamp::new(*i_r.clone(), *e_r.clone()).fill();
                        let new_e_l = EventTree::leaf(max(e_l.max(), eprime_r.min()));
                        EventTree::node(*n, Box::new(new_e_l), Box::new(eprime_r)).norm()
                    },
                    (_, IdTree::Leaf { i: true }) => {
                        let eprime_l = Stamp::new(*i_l.clone(), *e_l.clone()).fill();
                        let new_e_r = EventTree::leaf(max(e_r.max(), eprime_l.min()));
                        EventTree::node(*n, Box::new(eprime_l), Box::new(new_e_r)).norm()
                    },
                    (_, _) => {
                        let new_e_l = Stamp::new(*i_l.clone(), *e_l.clone()).fill();
                        let new_e_r = Stamp::new(*i_r.clone(), *e_r.clone()).fill();
                        EventTree::node(*n, Box::new(new_e_l), Box::new(new_e_r)).norm()
                    },
                }
            },
        }
    }

    pub fn grow(&self) -> (EventTree, Cost) {
        match (&self.i, &self.e) {
            (IdTree::Leaf { i: true }, EventTree::Leaf { n }) => (EventTree::leaf(*n+1), Cost::zero()),
            (_, EventTree::Leaf { n }) => {
                let new_e = EventTree::node(*n, Box::new(EventTree::zero()), Box::new(EventTree::zero()));
                let (eprime, c) = Stamp::new(self.i.clone(), new_e).grow();
                (eprime, c.shift())
            },
            // TODO: rewrite this as multiple cases instead of a second match
            //       statement if/once the box syntax is stabilised
            (IdTree::Node { left: i_l, right: i_r }, EventTree::Node { n, left: e_l, right: e_r }) => {
                match (i_l.as_ref(), i_r.as_ref()) {
                    (IdTree::Leaf { i: false }, _) => {
                        let (eprime_r, c_r) = Stamp::new(*i_r.clone(), *e_r.clone()).grow();
                        let new_e = EventTree::node(*n, e_l.clone(), Box::new(eprime_r));
                        (new_e, c_r+1)
                    },
                    (_, IdTree::Leaf { i: false }) => {
                        let (eprime_l, c_l) = Stamp::new(*i_l.clone(), *e_l.clone()).grow();
                        let new_e = EventTree::node(*n, Box::new(eprime_l), e_r.clone());
                        (new_e, c_l+1)
                    },
                    (_, _) => {
                        let (eprime_l, c_l) = Stamp::new(*i_l.clone(), *e_l.clone()).grow();
                        let (eprime_r, c_r) = Stamp::new(*i_r.clone(), *e_r.clone()).grow();
                        if c_l < c_r {
                            let new_e = EventTree::node(*n, Box::new(eprime_l), e_r.clone());
                            (new_e, c_l+1)
                        } else {
                            let new_e = EventTree::node(*n, e_l.clone(), Box::new(eprime_r));
                            (new_e, c_r+1)
                        }
                    },
                }
            },
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Stamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.e.partial_cmp(&other.e)
    }
}
