use std::{collections::BTreeSet, hash::Hash};

use automata::{
    ts::{HasInput, HasStates, IntoTransitions, LengthLexicographicEdges, Trivial, Visitor},
    words::{IsFinite, IsInfinite},
    Class, FiniteKind, Pointed, RightCongruence, Set, Subword, Successor, Symbol, TransitionSystem,
    UltimatelyPeriodicWord, Word, DFA,
};
use itertools::Itertools;
use tracing::trace;

/// Represents a finite sample, which is a pair of positive and negative instances.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct Sample<W> {
    pub positive: Set<W>,
    pub negative: Set<W>,
}

impl<W: Eq + Hash> PartialEq for Sample<W> {
    fn eq(&self, other: &Self) -> bool {
        self.positive == other.positive && self.negative == other.negative
    }
}

impl<W: Eq + Hash> Eq for Sample<W> {}

impl<W: IsInfinite> Sample<W> {
    /// Returns the maximum length of the base prefix of any word in the sample.
    pub fn max_base_len(&self) -> usize {
        self.iter().map(|w| w.base_length()).max().unwrap_or(0)
    }

    /// Returns the maximum loop length of any word in the sample.
    pub fn max_recur_len(&self) -> usize {
        self.iter().map(|w| w.recur_length()).max().unwrap_or(0)
    }
}

impl<W: Eq + Hash> Sample<W> {
    /// Creates a new sample from the given data.
    pub fn from_parts(positive: Set<W>, negative: Set<W>) -> Self {
        Self { positive, negative }
    }

    /// Returns the number of elements in the sample.
    pub fn len(&self) -> usize {
        self.positive.len() + self.negative.len()
    }

    /// Checks if the sample is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new sample from two iterators.
    pub fn from_iters<I, J>(positive: I, negative: J) -> Self
    where
        I: IntoIterator<Item = W>,
        J: IntoIterator<Item = W>,
    {
        Self {
            positive: positive.into_iter().collect(),
            negative: negative.into_iter().collect(),
        }
    }

    /// Iterates over all elements in the sample.
    pub fn iter(&self) -> impl Iterator<Item = &W> {
        self.positive.iter().chain(self.negative.iter())
    }

    /// Iterates just over the positive instances.
    pub fn positive_iter(&self) -> impl Iterator<Item = &W> {
        self.positive.iter()
    }

    /// Iterates just over the negative instances.
    pub fn negative_iter(&self) -> impl Iterator<Item = &W> {
        self.negative.iter()
    }

    /// Iterates over all elements in the sample.
    pub fn annotated_iter(&self) -> impl Iterator<Item = (bool, &W)> {
        self.positive
            .iter()
            .map(|w| (true, w))
            .chain(self.negative.iter().map(|w| (false, w)))
    }
}

impl<W: Word<Kind = FiniteKind>> Sample<W> {}

pub struct Prefixes<'a, S: Symbol> {
    alphabet: Set<S>,
    set: &'a Set<UltimatelyPeriodicWord<S>>,
}

impl<'a, S: Symbol> Prefixes<'a, S> {
    pub fn new(set: &'a Set<UltimatelyPeriodicWord<S>>) -> Self {
        let alphabet = set.iter().flat_map(|w| w.alphabet()).cloned().collect();

        Self { alphabet, set }
    }

    fn find_words_with_prefix(&self, class: &Class<S>) -> Vec<&UltimatelyPeriodicWord<S>> {
        self.set.iter().filter(|w| w.has_prefix(&class.0)).collect()
    }
}

impl<'a, S: Symbol> HasStates for Prefixes<'a, S> {
    type Q = Class<S>;
}

impl<'a, S: Symbol> HasInput for Prefixes<'a, S> {
    type Sigma = S;

    type Input<'me> = std::collections::hash_set::Iter<'me, S>
    where Self:'me;

    fn raw_input_alphabet_iter(&self) -> Self::Input<'_> {
        self.alphabet.iter()
    }
}

impl<'a, S: Symbol> Successor for Prefixes<'a, S> {
    fn successor<X: std::borrow::Borrow<Self::Q>, Y: std::borrow::Borrow<Self::Sigma>>(
        &self,
        from: X,
        on: Y,
    ) -> Option<Self::Q> {
        let source = from.borrow();
        let sym = on.borrow();
        let successor = source + sym;
        trace!(
            "Computing successor of {} on {}, candidate is {}",
            source,
            sym,
            successor
        );

        let words_with_prefix = self.find_words_with_prefix(&successor);
        if words_with_prefix.is_empty() {
            trace!("No words with prefix {}!", successor);
            return None;
        }
        let count_words_with_prefix = words_with_prefix.len();
        trace!(
            "Found {} words with prefix {}",
            count_words_with_prefix,
            successor
        );

        if words_with_prefix.len() > 1 {
            Some(successor)
        } else {
            assert!(words_with_prefix.len() == 1);
            let word = words_with_prefix[0];
            let base = successor.prefix(successor.length() - word.recur_length());

            let words_with_prefix_for_base = self.find_words_with_prefix(&base);
            debug_assert!(!words_with_prefix_for_base.is_empty(), "Cannot happen!");

            if words_with_prefix_for_base.len() == 1 {
                Some(base)
            } else if !words_with_prefix_for_base.is_empty() {
                Some(successor)
            } else {
                None
            }
        }
    }
}

impl<'a, S: Symbol> Pointed for Prefixes<'a, S> {
    fn initial(&self) -> Self::Q {
        Class::epsilon()
    }
}

pub fn prefix_acceptor<S: Symbol>(set: &Set<UltimatelyPeriodicWord<S>>) -> DFA<Class<S>, S> {
    let prefixes = Prefixes::new(set);
    DFA::all_accepting_iters(
        LengthLexicographicEdges::new(&prefixes).iter(),
        Class::epsilon(),
    )
}

impl<S: Symbol> Sample<UltimatelyPeriodicWord<S>> {
    pub fn build_separated(&self) -> Self {
        let mut pos = Set::new();
        let mut neg = Set::new();

        let mut queue = self.annotated_iter().collect_vec();
        while let Some((is_positive, word)) = queue.pop() {
            let mut word = word.clone();
            while queue.iter().any(|(_, w)| w.base() == word.base()) {
                word.unroll_one();
            }
            if is_positive {
                pos.insert(word.clone());
            } else {
                neg.insert(word.clone());
            }
        }

        Sample::from_parts(pos, neg)
    }
}

impl<S: Symbol> Sample<UltimatelyPeriodicWord<S>> {
    pub fn positive_prefixes(&self) -> DFA<Class<S>, S> {
        todo!()
    }

    pub fn negative_prefixes(&self) -> DFA<Class<S>, S> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use automata::{
        ts::{LengthLexicographicEdges, Visitor},
        upw, Accepts, Class, Set, UltimatelyPeriodicWord, DFA,
    };
    use tracing::trace;

    use super::Prefixes;

    #[test]
    #[tracing_test::traced_test]
    fn set_prefixes_automaton() {
        trace!("set_prefixes_automaton");
        let words = Set::from_iter([upw!("a"), upw!("b"), upw!("ab")]);

        let prefix_dfa = super::prefix_acceptor(&words);

        for p in [
            "",
            "a",
            "b",
            "ab",
            "aaaaaaaaaa",
            "abababababab",
            "abababababababa",
        ] {
            assert!(prefix_dfa.accepts(p));
        }
        for n in ["bba", "aab", "abb", "abababbabababaa"] {
            assert!(!prefix_dfa.accepts(n));
        }
    }
}
