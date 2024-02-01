use std::collections::BTreeSet;

use crate::{
    algorithms::moore_partition_refinement,
    prelude::*,
    ts::{
        finite::ReachedColor,
        operations::{MapStateColor, MatchingProduct},
        Quotient,
    },
};

use super::{acceptor::FiniteWordAcceptor, AsMooreMachine, StatesWithColor};

impl_moore_automaton! {
    /// A deterministic finite automaton consists of a finite set of states, a finite set of input
    /// symbols, a transition function, a start state, and a set of accepting states.
    /// Internally, it is represented as a [`MooreMachine`], i.e. a transition system for which we consider
    /// the `bool` values that it has on the states. So a DFA accepts an input (which is the same as
    /// the Moore machine outputs `true` on the input), if the value of the state that it reaches upon
    /// reading the input is `true`.
    DFA, bool
}

impl<D: DFALike> IntoDFA<D> {
    fn separate<X, Y>(&self, left: X, right: Y) -> Option<Vec<SymbolOf<Self>>>
    where
        X: Indexes<Self>,
        Y: Indexes<Self>,
    {
        let q = left.to_index(self)?;
        let p = right.to_index(self)?;
        if p == q {
            return None;
        }

        self.with_initial(q)
            .ts_product(self.with_initial(p))
            .minimal_representatives()
            .find_map(|(rep, ProductIndex(l, r))| {
                if self.state_color(l).unwrap() != self.state_color(r).unwrap() {
                    Some(rep)
                } else {
                    None
                }
            })
    }
}

impl<Ts> FiniteWordAcceptor<SymbolOf<Self>> for DFA<Ts::Alphabet, Ts::EdgeColor, Ts>
where
    Ts: DFALike,
{
    fn accepts_finite<W: FiniteWord<SymbolOf<Self>>>(&self, word: W) -> bool {
        self.reached_state_color(word).unwrap_or(false)
    }
}

/// Helper trait to convert from boolean to usize. Normally, a `true` value corresponds to `1`, while
/// a `false` value corresponds to `0`. This does not really work well with min-even parity conditions
/// so this helper trait is introduced.
// TODO: remove this if possible.
pub trait ReducesTo<T = bool> {
    /// Reduce `self` to a value of type `T`.
    fn reduce(self) -> T;
}

impl ReducesTo<bool> for bool {
    fn reduce(self) -> bool {
        self
    }
}

impl ReducesTo<bool> for usize {
    fn reduce(self) -> bool {
        (self % 2) == 0
    }
}

impl ReducesTo<bool> for BTreeSet<bool> {
    fn reduce(self) -> bool {
        self.into_iter().any(|x| x)
    }
}

impl ReducesTo<bool> for BTreeSet<usize> {
    fn reduce(self) -> bool {
        self.into_iter().min().unwrap() % 2 == 0
    }
}

type DfaProductReduced<L, R> = MapStateColor<MatchingProduct<L, R>, fn((bool, bool)) -> bool>;

/// This trait is (automatically) implemented by everything which can be viewed as a [`DFA`].
pub trait DFALike: Deterministic<StateColor = bool> + Pointed
// + Acceptor<SymbolOf<Self>, FiniteLength>
// + Transformer<SymbolOf<Self>, FiniteLength, Output = bool>
{
    /// Consumes and turns `self` into a [`DFA`].
    fn into_dfa(self) -> DFA<Self::Alphabet, Self::EdgeColor, Self> {
        DFA::from(self)
    }

    /// Consumes and turns `self` into a [`DFA`]. Note, that this operation erases the edge colors.
    fn collect_dfa(self) -> DFA<Self::Alphabet> {
        DFA::from(self.erase_edge_colors().collect_with_initial())
    }

    /// Uses a reference to `self` for creating a [`DFA`].
    fn as_dfa(&self) -> DFA<Self::Alphabet, Self::EdgeColor, &Self> {
        DFA::from(self)
    }

    /// Returns the indices of all states that are accepting.
    fn accepting_states(&self) -> StatesWithColor<'_, Self> {
        StatesWithColor::new(self, true)
    }

    /// Returns the indices of all states that are rejecting.
    fn rejecting_states(&self) -> StatesWithColor<'_, Self> {
        StatesWithColor::new(self, false)
    }

    /// Minimizes `self` using Hopcroft's partition refinement algorithm.
    fn dfa_minimized(self) -> IntoDFA<AsMooreMachine<Self>> {
        let min = moore_partition_refinement(self);
        min.into_dfa()
    }

    /// Checks whether `self` is equivalent to `other`, i.e. whether the two DFAs accept
    /// the same language. This is done by negating `self` and then verifying that the intersection
    /// of the negated automaton with `other` is empty.
    fn equivalent<D: DFALike<Alphabet = Self::Alphabet>>(&self, other: D) -> bool {
        self.negation()
            .intersection(other)
            .dfa_give_word()
            .is_none()
    }

    /// Tries to construct a (finite) word witnessing that the accepted language is empty. If such a word exists,
    /// the function returns it, otherwise `None`.
    fn dfa_give_word(&self) -> Option<Vec<SymbolOf<Self>>> {
        self.minimal_representatives().find_map(|(mr, index)| {
            if self
                .state_color(index)
                .expect("Every state must be colored")
            {
                Some(mr)
            } else {
                None
            }
        })
    }

    /// Returns true if and only if the accepted language is empty.
    fn dfa_is_empty(&self) -> bool {
        self.dfa_give_word().is_none()
    }

    /// Computes the union of `self` with the given `other` object (that can be viewed as a DFA) through
    /// a simple product construction.
    fn union<Ts: DFALike<Alphabet = Self::Alphabet>>(
        self,
        other: Ts,
    ) -> DfaProductReduced<Self, Ts> {
        self.ts_product(other).map_state_colors(|(a, b)| a || b)
    }

    /// Computes the intersection of `self` with the given `other` object (that can be viewed as a DFA) through
    /// a simple product construction.
    fn intersection<Ts: DFALike<Alphabet = Self::Alphabet>>(
        self,
        other: Ts,
    ) -> DfaProductReduced<Self, Ts> {
        self.ts_product(other).map_state_colors(|(a, b)| a && b)
    }

    /// Computes the negation of `self` by swapping accepting and non-accepting states.
    fn negation(self) -> MapStateColor<Self, fn(bool) -> bool> {
        self.map_state_colors(|x| !x)
    }
}

impl<Ts> DFALike for Ts where Ts: Deterministic<StateColor = bool> + Pointed + Sized {}
