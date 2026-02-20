//! # Interval Tree Clocks
//!
//! The itc crate implements Interval Tree Clocks as described in
//! http://gsd.di.uminho.pt/members/cbm/ps/itc2008.pdf
//!
//! # Usage:
//!
//! ```
//! use itc::*;
//!
//! let s = Stamp::seed();
//!
//! let (s1, s2) = s.fork();
//! let s1prime = s1.event();
//! let s2prime = s2.event();
//! let s3 = s2prime.join(&s1);
//!
//! assert!(s.leq(&s1));
//! assert!(s1.leq(&s1prime));
//! assert!(!s1prime.leq(&s2prime));
//! assert!(s2prime.leq(&s3));
//! ```
//!
//! This module implements the 4 verbs: fork, event, join, peek,
//! the 3 derived verbs: send, receive and sync,
//! and a partial ordering to establish causality / the happens-before relation.
//! Also in the box is a simple ascii coding representation suitable
//! for printing to stdout and use in protocols.


pub mod ascii_coding;
pub mod bin_coding;
pub mod cost;
pub mod event_tree;
pub mod id_tree;
pub mod interval_tree_clock;
pub mod normalisable;
pub mod stamp;


#[cfg(test)]
mod tests {
    use crate::event_tree::EventTree;
    use crate::id_tree::{IdTree, Split};
    use crate::interval_tree_clock::IntervalTreeClock;
    use crate::normalisable::Normalisable;
    use crate::stamp::Stamp;

    #[test]
    fn norm_id_one_is_one() {
        let idt = IdTree::one();
        let nidt = idt.norm();
        assert_eq!(nidt, IdTree::one());
    }

    #[test]
    fn norm_id_zero_is_zero() {
        let idt = IdTree::zero();
        let nidt = idt.norm();
        assert_eq!(nidt, IdTree::zero());
    }

    #[test]
    fn norm_id_0_0_is_0() {
        let idt = IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::zero()));
        let nidt = idt.norm();
        assert_eq!(nidt, IdTree::zero());
    }

    #[test]
    fn norm_id_1_1_is_1() {
        let idt = IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::one()));
        let nidt = idt.norm();
        assert_eq!(nidt, IdTree::one());
    }

    #[test]
    fn norm_id_0_1_is_0_1() {
        let idt = IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero()));
        let nidt = idt.clone().norm();
        assert_eq!(nidt, idt);
    }

    #[test]
    fn norm_id_1_1_1_is_1() {
        let idt = IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::one()))));
        let nidt = idt.clone().norm();
        assert_eq!(nidt, IdTree::one());
    }

    // (2, 1, 1) ~=~ 3
    #[test]
    fn norm_e_one() {
        let et = EventTree::node(2, Box::new(EventTree::leaf(1)), Box::new(EventTree::leaf(1)));
        let net = et.clone().norm();
        assert_eq!(net, EventTree::leaf(3));
    }

    // (2, (2, 1, 0), 3) ~=~ (4, (0, 1, 0), 1)
    #[test]
    fn norm_e_two() {
        let a = Box::new(EventTree::node(2, Box::new(EventTree::leaf(1)), Box::new(EventTree::leaf(0))));
        let b = Box::new(EventTree::leaf(3));
        let et = EventTree::node(2, a, b);

        let expected_a = Box::new(EventTree::node(0, Box::new(EventTree::leaf(1)), Box::new(EventTree::leaf(0))));
        let expected_b = Box::new(EventTree::leaf(1));
        let expected = EventTree::node(4, expected_a, expected_b);

        let net = et.norm();

        assert_eq!(net, expected);
    }

    #[test]
    fn split_test() {
        assert_eq!(IdTree::one().split(), (IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one()))));
    }

    #[test]
    fn example() {
        let seed = Stamp::seed();
        let (l, r) = seed.fork();

        assert_eq!(l, Stamp::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), EventTree::zero()));
        assert_eq!(r, Stamp::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one())), EventTree::zero()));

        let le = l.event();
        let re = r.event();

        assert_eq!(le, Stamp::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), EventTree::node(0, Box::new(EventTree::leaf(1)), Box::new(EventTree::zero()))));
        assert_eq!(re, Stamp::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one())), EventTree::node(0, Box::new(EventTree::zero()), Box::new(EventTree::leaf(1)))));

        let (lel, ler) = le.fork();

        assert_eq!(lel, Stamp::new(IdTree::node(Box::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero()))), Box::new(IdTree::zero())), EventTree::node(0, Box::new(EventTree::leaf(1)), Box::new(EventTree::zero()))));
        assert_eq!(ler, Stamp::new(IdTree::node(Box::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one()))), Box::new(IdTree::zero())), EventTree::node(0, Box::new(EventTree::leaf(1)), Box::new(EventTree::zero()))));

        let ree = re.event();

        assert_eq!(ree, Stamp::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one())), EventTree::node(0, Box::new(EventTree::zero()), Box::new(EventTree::leaf(2)))));

        let lele = lel.event();

        assert_eq!(lele, Stamp::new(IdTree::node(Box::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero()))), Box::new(IdTree::zero())), EventTree::node(0, Box::new(EventTree::node(1, Box::new(EventTree::leaf(1)), Box::new(EventTree::zero()))), Box::new(EventTree::zero()))));

        let lerjree = ler.join(&ree);

        assert_eq!(lerjree, Stamp::new(IdTree::node(Box::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one()))), Box::new(IdTree::one())), EventTree::node(1, Box::new(EventTree::zero()), Box::new(EventTree::leaf(1)))));

        let (lerjreel, lerjreer) = lerjree.fork();

        assert_eq!(lerjreel, Stamp::new(IdTree::node(Box::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one()))), Box::new(IdTree::zero())), EventTree::node(1, Box::new(EventTree::zero()), Box::new(EventTree::leaf(1)))));
        assert_eq!(lerjreer, Stamp::new(IdTree::node(Box::new(IdTree::zero()), Box::new(IdTree::one())), EventTree::node(1, Box::new(EventTree::zero()), Box::new(EventTree::leaf(1)))));

        let lelejlerjreel = lele.join(&lerjreel);

        assert_eq!(lelejlerjreel, Stamp::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), EventTree::node(1, Box::new(EventTree::node(0, Box::new(EventTree::leaf(1)), Box::new(EventTree::zero()))), Box::new(EventTree::leaf(1)))));

        let lelejlerjreele = lelejlerjreel.event();

        assert_eq!(lelejlerjreele, Stamp::new(IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())), EventTree::leaf(2)));
    }

    #[test]
    fn bin_encoding() {
        let (stamp, _) = Stamp::seed()
            .event()
            .event()
            .fork();
        let stamp = stamp.event()
            .event();
        let b: Box<[u8]> = stamp.into();
        println!("{:?}", b);
    }
}
