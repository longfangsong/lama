use std::collections::VecDeque;

use automata::{
    ts::{FiniteState, Product, Sproutable},
    Alphabet, InfiniteLength, Pointed, RightCongruence, Successor,
};
use itertools::Itertools;
use tracing::trace;

use crate::{prefixtree::prefix_tree, Sample};

#[derive(Clone)]
pub struct ConflictRelation<A: Alphabet> {
    dfas: [RightCongruence<A>; 2],
    conflicts: Vec<(usize, usize)>,
}

impl<A: Alphabet> ConflictRelation<A> {
    pub fn consistent(&self, cong: &RightCongruence<A>) -> bool {
        let left = cong.product(&self.dfas[0]);
        let right = cong.product(&self.dfas[1]);

        if left.reachable_state_indices().any(|left_index| {
            right
                .reachable_state_indices()
                .any(|right_index| self.conflicts.contains(&(left_index.1, right_index.1)))
        }) {
            return false;
        }
        true
    }
}

fn prefix_consistency_conflicts<A: Alphabet>(
    alphabet: &A,
    sample: Sample<A, InfiniteLength, bool>,
) -> ConflictRelation<A> {
    let left_pta = prefix_tree(alphabet.clone(), sample.positive_words().cloned().collect());
    let right_pta = prefix_tree(alphabet.clone(), sample.negative_words().cloned().collect());

    let dfa = (&left_pta).product(&right_pta);
    let conflicts: Vec<(usize, usize)> = dfa
        .sccs()
        .into_iter()
        .filter_map(|scc| {
            if !scc.is_trivial() {
                Some(scc.into_iter().map(Into::into))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    ConflictRelation {
        dfas: [left_pta, right_pta],
        conflicts,
    }
}

pub fn omega_sprout<A: Alphabet>(
    alphabet: A,
    conflicts: ConflictRelation<A>,
) -> RightCongruence<A> {
    let mut cong = RightCongruence::new(alphabet.clone());
    let initial = cong.initial();

    // We maintain a set of missing transitions and go through them in order of creation for the states and in order
    // give by alphabet for the symbols for one state (this amouts to BFS).
    let mut queue: VecDeque<_> = alphabet.universe().map(|sym| (initial, sym)).collect();
    'outer: while let Some((source, &sym)) = queue.pop_front() {
        trace!("Trying to add transition from {} on {:?}", source, sym);
        for target in cong.state_indices() {
            cong.add_edge(source, A::expression(sym), target, ());

            if conflicts.consistent(&cong) {
                continue 'outer;
            } else {
                trace!("\tTransition to {target} is not consistent");
                cong.undo_add_edge();
            }
        }

        let mut new_state_label = cong.state_color(source).clone();
        new_state_label.push(sym);
        trace!(
            "No consistent transition found, adding new state [{}]",
            new_state_label.iter().map(|c| format!("{:?}", c)).join("")
        );

        let new_state = cong.add_state(new_state_label);
        queue.extend(std::iter::repeat(new_state).zip(alphabet.universe()))
    }

    cong
}

#[cfg(test)]
mod tests {
    use automata::{
        simple,
        ts::{
            finite::{ReachedColor, ReachedState},
            FiniteState, Sproutable,
        },
        Pointed, RightCongruence, Successor,
    };
    use tracing_test::traced_test;

    use crate::Sample;

    #[test]
    #[traced_test]
    fn prefix_consistency_sprout() {
        #[test]
        fn prefix_consistency() {
            let alphabet = simple!('a', 'b');
            let sample = Sample::new_omega(
                alphabet.clone(),
                vec![
                    ("b", 0, true),
                    ("abab", 3, true),
                    ("abbab", 4, true),
                    ("ab", 1, false),
                    ("a", 0, false),
                ],
            );
            let mut expected_cong = RightCongruence::new(simple!('a', 'b'));
            let q0 = expected_cong.initial();
            let q1 = expected_cong.add_state(vec!['a']);
            expected_cong.add_edge(q0, 'a', q1, ());
            expected_cong.add_edge(q1, 'a', q0, ());
            expected_cong.add_edge(q0, 'b', q0, ());
            expected_cong.add_edge(q1, 'b', q1, ());

            let conflicts = super::prefix_consistency_conflicts(&alphabet, sample);
            let cong = super::omega_sprout(alphabet, conflicts);

            assert_eq!(cong.size(), expected_cong.size());
            for word in ["aba", "abbabb", "baabaaba", "bababaaba", "b", "a", ""] {
                assert_eq!(
                    cong.reached_color(&word),
                    expected_cong.reached_color(&word)
                )
            }
        }
    }
}
