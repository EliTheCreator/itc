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


#[cfg(test)]
mod tests {
    use crate::event_tree::EventTree;
    use crate::id_tree::IdTree;
    use crate::normalisable::Normalisable;

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
}
