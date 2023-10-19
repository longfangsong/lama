use std::fmt::Debug;

use automata::prelude::*;

use crate::active::{oracle::MealyOracle, LStar};

use super::Sample;

type DFASample<D> = Sample<<D as HasAlphabet>::Alphabet, FiniteLength, bool>;

pub fn characterize_dfa<D: DFALike>(dfa: D) -> DFASample<D> {
    todo!()
}

pub fn actively_exchanged_words_dfa<D: DFALike>(dfa: D) -> DFASample<D> {
    todo!()
}

type MealySample<D> =
    Sample<<D as HasAlphabet>::Alphabet, FiniteLength, <D as TransitionSystem>::EdgeColor>;

pub fn actively_exchanged_words_mealy<C: Color + Default + Debug, D: MealyLike<C>>(
    mm: D,
) -> MealySample<D> {
    let alphabet = mm.alphabet().clone();
    let oracle = MealyOracle::new(mm);
    let mut lstar = LStar::logged(oracle, alphabet);
    // let learned = lstar.learn();
    // let mut sample = MealySample::new(alphabet);
    todo!()
}
