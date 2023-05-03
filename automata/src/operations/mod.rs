use std::borrow::Borrow;

pub use crate::TransitionOutput;
use crate::{Equivalent, Pair, StateIndex, Symbol, DFA};

mod mapping;

mod product;
pub use product::{product_transitions, Product};

mod trimming;

impl<Q: StateIndex, S: Symbol> DFA<Q, S> {
    /// Computes the union of two DFAs. This is built upon the [`direct_product`], where the acceptance
    /// condition is computed by taking the disjunction of the two acceptance conditions.
    pub fn union<P, D>(&self, other: &D) -> DFA<Pair<Q, P>, S>
    where
        P: StateIndex,
        D: Borrow<DFA<P, S>>,
    {
        self.direct_product(other.borrow())
            .map_acceptance(|x| x.left || x.right)
    }

    /// Computes the intersection of two DFAs. We build this through the [`direct_product`] and obtain
    /// the acceptance by taking the conjunction of the two acceptance conditions.
    pub fn intersection<P, D>(&self, other: &D) -> DFA<Pair<Q, P>, S>
    where
        P: StateIndex,
        D: Borrow<DFA<P, S>>,
    {
        self.direct_product(other.borrow())
            .map_acceptance(|x| x.left && x.right)
    }

    /// Builds the negation of the DFA by inverting/negating the acceptance condition.
    pub fn negation(&self) -> DFA<Q, S> {
        self.clone().map_acceptance(|x| !x)
    }
}

impl<T: TransitionOutput> Equivalent for T {
    fn equivalent(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{acceptance::Accepts, DFA};

    #[test]
    fn dfa_operations() {
        let left = DFA::from_parts_iters(
            [
                (0, 'a', 1),
                (1, 'a', 2),
                (2, 'a', 0),
                (0, 'b', 0),
                (1, 'b', 1),
                (2, 'b', 2),
            ],
            [0],
            0,
        );
        let right =
            DFA::from_parts_iters([(0, 'a', 0), (1, 'a', 1), (0, 'b', 1), (1, 'b', 0)], [0], 0);

        let union = left.union(&right);
        for p in ["aaa", "bb", "abb", "b"] {
            assert!(union.accepts(p));
        }
        for n in ["ab", "aba", "baa", "aababba"] {
            assert!(!union.accepts(n));
        }

        let intersection = left.intersection(&right);
        for p in ["", "aaabb", "bb", "aaa"] {
            assert!(intersection.accepts(p));
        }
        for n in ["a", "b", "aba", "bba", "aaab", "baabab"] {
            assert!(!intersection.accepts(n));
        }

        let negation = left.negation();
        for p in ["a", "aa", "ba"] {
            assert!(negation.accepts(p));
        }
        for n in ["", "aaa"] {
            assert!(!negation.accepts(n));
        }
    }
}
