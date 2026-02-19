use crate::{id_tree::{IdTree, Split, Sum}, stamp::Stamp};


pub trait IntervalTreeClock where Self: Sized {
    fn fork(&self) -> (Self, Self);
    fn peek(&self) -> (Self, Self);
    fn join(&self, other: &Self) -> Self;
    fn event(&self) -> Self;

    fn send(&self) -> (Self, Self);
    fn receive(&self, other: &Self) -> Self;
    fn sync(&self, other: &Self) -> (Self, Self);
}

impl IntervalTreeClock for Stamp {
    fn peek(&self) -> (Stamp, Stamp) {
        let s1 = Stamp::new(IdTree::zero(), self.e.clone());
        let s2 = Stamp::new(self.i.clone(), self.e.clone());
        return (s1, s2)
    }

    fn fork(&self) -> (Stamp, Stamp) {
        if let IdTree::Node {left, right} = self.i.split() {
            let s1 = Stamp::new(*left, self.e.clone());
            let s2 = Stamp::new(*right, self.e.clone());
            (s1, s2)
        } else {
            unreachable!()
        }
    }

    fn join(&self, other: &Stamp) -> Stamp {
        let sum_i = self.i.sum(&other.i);
        let join_e = self.e.join(&other.e);
        Stamp::new(sum_i, join_e)
    }

    fn event(&self) -> Stamp {
        let filled_e = self.fill();

        if filled_e.as_ref() != &self.e {
            Stamp::new(self.i.clone(), filled_e.into_owned())
        } else {
            let (eprime, _c) = self.grow();

            Stamp::new(self.i.clone(), eprime)
        }
    }

    fn send(&self) -> (Stamp, Stamp) {
        self.event().peek()
    }

    fn receive(&self, other: &Stamp) -> Stamp {
        self.join(other).event()
    }

    fn sync(&self, other: &Stamp) -> (Stamp, Stamp) {
        self.join(other).fork()
    }
}
