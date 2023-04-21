use std::{borrow::Borrow, fmt::Display};

use crate::{run::Configuration, Set, Symbol, Word};

mod implementations;
mod initialized;
mod transition;
mod visit;

use impl_tools::autoimpl;
use owo_colors::OwoColorize;
use tabled::{builder::Builder, settings::Style};
pub use transition::{Transition, Trigger};

/// An implementation of a deterministic `TransitionSystem` in form of an edge list. The edge list is represented by a vector of tuples `(from, to, symbol)`. Is only available if the `det` feature is enabled.
#[cfg(feature = "det")]
pub mod deterministic;
#[cfg(feature = "det")]
pub use deterministic::{Deterministic, InitializedDeterministic};

use self::visit::{LengthLexicographic, LengthLexicographicEdges};

/// A trait for the state index type. Implementors must be comparable, hashable, clonable and debuggable. The `create` method is used to create a new state index from a `u32`
pub trait StateIndex: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug + Ord {}

impl<X: Clone + Eq + PartialEq + std::hash::Hash + std::fmt::Debug + Ord> StateIndex for X {}

// The following two type aliases might change in the future to allow for more flexibility, i.e. for example for implementing nondeterminism.
/// Helper type for getting the symbol type of a transition system.
pub type SymbolOf<X> = <X as TransitionSystem>::Input;
/// Helper type for getting the output type of a transition system.
pub type StateOf<X> = <X as TransitionSystem>::State;
/// Helper type for getting the trigger type of a transition system.
pub type TriggerOf<TS> = (StateOf<TS>, SymbolOf<TS>);
/// Helper type for getting the transition type of a transition system.
pub type TransitionOf<TS> = (StateOf<TS>, SymbolOf<TS>, StateOf<TS>);

/// The base trait implemented by a deterministic transition system. A transition system is a tuple `(Q, S, δ)`, where `Q` is a finite set of states, `S` is a finite set of symbols and `δ: Q × S → Q` is a transition function. Note that the transition function is not necessarily complete and some transitions may be missing.
/// States of a transition system are generic, and can be any type that implements the [`StateIndex`] trait.
/// Also the symbols of a transition system are generic, and can be any type that implements the [`Alphabet`] trait.
/// The [`TransitionTrigger`] trait is used to represent an outgoing transition. Note, that such a trigger is a pair consisting of a state and a symbol, meaning the target state is not included in a trigger.
#[autoimpl(for<T: trait> &T, &mut T, initialized::Initialized<T>)]
pub trait TransitionSystem {
    /// The type of the states of the transition system.
    type State: StateIndex;
    /// The type of the symbols of the transition system.
    type Input: Symbol;

    /// Returns the successor state of the given state on the given symbol. The transition function is deterministic, meaning that if a transition exists, it is unique. On the other hand there may not be a transition for a given state and symbol, in which case `succ` returns `None`.
    fn succ(&self, from: &Self::State, on: &Self::Input) -> Option<Self::State>;

    /// Returns the successor state for the given trigger through calling [`Self::succ`].
    fn apply_trigger(&self, trigger: &(Self::State, Self::Input)) -> Option<Self::State> {
        self.succ(trigger.source(), trigger.sym())
    }

    /// Creates a new trigger from the given state and symbol.
    fn make_trigger(from: &Self::State, on: &Self::Input) -> (Self::State, Self::Input) {
        (from.clone(), on.clone())
    }

    /// Starts a run from the given state. A run is given by a [`Configuration`] object, which keeps track
    /// of the current state.
    fn run_word_from<W: Word<S = Self::Input>>(
        &self,
        on: W,
        from: Self::State,
    ) -> Configuration<&Self, W>
    where
        Self: Sized,
    {
        Configuration::from_state(self, from, on)
    }

    /// Creates a copy of the current TS which has its initial state set.
    fn start(&self, start: Self::State) -> initialized::Initialized<Self>
    where
        Self: Sized + Clone,
    {
        initialized::Initialized {
            ts: self.clone(),
            start,
        }
    }

    /// Returns the set of all states of the transition system. This is costly.
    fn vec_alphabet(&self) -> Vec<Self::Input>;

    /// Returns the set of all symbols of the transition system. This is costly.
    fn vec_states(&self) -> Vec<Self::State>;

    /// Builds a string representation of the transition table of the transition system.
    /// For this, the [`tabled`] crate is used.
    fn display_transition_table(&self) -> String
    where
        Self::State: Display,
    {
        let mut builder = Builder::default();
        builder.set_header(
            vec!["Deterministic".to_string()].into_iter().chain(
                self.vec_alphabet()
                    .into_iter()
                    .map(|s| s.purple().to_string())
                    .collect::<Vec<String>>(),
            ),
        );
        for state in self.vec_states() {
            let mut row = vec![state.to_string()];
            for sym in self.vec_alphabet() {
                row.push(if let Some(successor) = self.succ(&state, &sym) {
                    successor.to_string()
                } else {
                    "-".to_string()
                });
            }
            builder.push_record(row);
        }
        let mut transition_table = builder.build();
        transition_table.with(Style::psql());
        transition_table.to_string()
    }

    /// Performs a breadth-first search on the transition system, starting from the given state.
    fn bfs_from(&self, start: Self::State) -> LengthLexicographic<&Self>
    where
        Self: Sized,
    {
        LengthLexicographic::new_from(self, start)
    }

    /// Performs a breadth-first search on the transition system, starting from the initial state.
    fn bfs(&self) -> LengthLexicographic<&Self>
    where
        Self: Sized + Pointed,
    {
        LengthLexicographic::new(self)
    }

    /// Performs a breadth-first search on the transition system, starting from the given state, emitting each visited edge.
    fn bfs_edges_from(&self, start: Self::State) -> LengthLexicographicEdges<&Self>
    where
        Self: Sized,
    {
        LengthLexicographicEdges::new_from(self, start)
    }

    /// Performs a breadth-first search on the transition system, starting from the initial state, emitting each visited edge.
    fn bfs_edges(&self) -> LengthLexicographicEdges<&Self>
    where
        Self: Sized + Pointed,
    {
        LengthLexicographicEdges::new(self)
    }
}

/// Creates a new trivial transition system, which could either be empty (for [`TransitionSystem`]) or contain a single initial state (for [`InitializedDeterministic`]).
pub trait Trivial: TransitionSystem {
    /// Creates the trivial object
    fn trivial() -> Self;
}

/// Implemented by objects which have a designated initial state.
#[autoimpl(for<T: trait> &T, &mut T)]
pub trait Pointed: TransitionSystem {
    /// Get the initial state of the automaton.
    fn initial(&self) -> Self::State;

    /// Runs the word from the given initial state.
    fn run<'t, W>(&'t self, on: &'t W) -> Configuration<&'t Self, &'t W>
    where
        Self: Sized,
        W: Word<S = Self::Input>,
    {
        Configuration::for_pointed(self, on)
    }
}

/// Implemented by objects which have a finite number of states.
pub trait StateIterable: TransitionSystem {
    /// Type of the iterator over all states.
    type StateIter<'me>: Iterator<Item = &'me Self::State>
    where
        Self: 'me;

    /// Returns a `Vec` of all states in the transition system.
    fn states_iter(&self) -> Self::StateIter<'_>;
}

/// Trait that allows iterating over all edges in a [`TransitionSystem`].
#[autoimpl(for<T: trait> &T, &mut T)]
pub trait TransitionIterable: TransitionSystem {
    /// Type of the iterator over all edges.
    type TransitionIter<'me>: Iterator<Item = &'me (Self::State, Self::Input, Self::State)>
    where
        Self: 'me,
        Self::State: 'me,
        Self::Input: 'me;

    /// Returns an iterator over all edges in the transition system.
    fn transitions_iter(&self) -> Self::TransitionIter<'_>;

    /// Returns the set of transitions originating from a given state.
    fn transitions_from(&self, from: &Self::State) -> Set<(Self::State, Self::Input, Self::State)> {
        self.transitions_iter()
            .filter(|e| e.source() == from)
            .cloned()
            .collect()
    }
}

/// Trait that allows iterating over all triggers in a [`TransitionSystem`].
pub trait TriggerIterable: TransitionSystem {
    /// THe iterator type
    type TriggerIter<'me>: Iterator<Item = &'me (Self::State, Self::Input)>
    where
        Self: 'me;

    /// Returns an iterator over all triggers in the transition system.
    fn triggers_iter(&self) -> Self::TriggerIter<'_>;

    /// Returns the set of triggers originating from a given state.
    fn triggers_from(&self, from: &Self::State) -> Set<&'_ (Self::State, Self::Input)> {
        self.triggers_iter()
            .filter(|e| e.source() == from)
            .collect()
    }
}

/// Converts the given transition system in to an Iterator over references to its states.
pub trait IntoStateReferences<'a>: TransitionSystem + 'a {
    /// The type of the iterator.
    type Output: Iterator<Item = &'a Self::State>;
    /// Converts the transition system into an iterator over references to its states.
    fn into_state_references(self) -> Self::Output;
}

/// Ecapsulates the ability to add states and transitions to a transition system.
pub trait Growable: TransitionSystem {
    /// Add a new state to the transition system..
    fn add_state(&mut self, state: &Self::State) -> bool;

    /// Add a new transition to the transition system. If the transition did not exist before, `None` is returned. Otherwise, the old target state is returned.
    fn add_transition<X: Borrow<Self::State>, Y: Borrow<Self::State>>(
        &mut self,
        from: X,
        on: SymbolOf<Self>,
        to: Y,
    ) -> Option<Self::State>;
}

/// Ecapsulates the ability to add anonymous states and transitions to a transition system.
pub trait AnonymousGrowable: Growable {
    /// Add a new state to the transition system..
    fn add_new_state(&mut self) -> Self::State;
}

/// Implmenetors of this trait can be shrunk, i.e. states and transitions can be removed from the transition system.
pub trait Shrinkable: TransitionSystem {
    /// Deletes the given state from the transition system. If the state did not exist before, `None` is returned. Otherwise, the old state is returned.
    /// This method does not remove any transitions which point to the given state.
    fn remove_state(&mut self, state: Self::State) -> Option<Self::State>;

    /// Deletes the given transition from the transition system. If the transition did not exist before, `None` is returned. Otherwise, the old target state is returned.
    fn remove_transition(&mut self, from: Self::State, on: SymbolOf<Self>) -> Option<Self::State>;
}

/// A trait implemented by a [`TransitionSystem`] which can be trimmed. This means that all unreachable states are removed from the transition system. Further, all transitions which point to or originate from unreachable states are removed. Note that this operation is only applicable to a [`TransitionSystem`] which is [`Pointed`], as the concept of reachability is only defined if a designated initial state is given.
pub trait Trimable: TransitionSystem + Pointed {
    /// Removes all unreachable states from the transition system. Additionally removes any transitions which point to or originate from unreachable states.
    fn trim(&mut self);
}
