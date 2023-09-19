/// Contains the most central trait for this module, the trait [`TransitionSystem`].
pub mod transition_system;
use std::{fmt::Display, hash::Hash, ops::Deref};

use impl_tools::autoimpl;
use itertools::Itertools;
pub use transition_system::TransitionSystem;

/// Defines implementations for common operations on automata/transition systems.
pub mod operations;

use crate::{Class, Color, Map, RightCongruence};

mod index_ts;
pub use index_ts::BTS;

/// Contains implementations and definitions for dealing with paths through a transition system.
pub mod path;
pub use path::Path;

mod sproutable;
pub use sproutable::Sproutable;

mod induces;
pub use induces::{finite, infinite, CanInduce, Induced};

/// Deals with analysing reachability in transition systems.
pub mod reachable;

/// Contains implementations for SCC decompositions and the corresponding/associated types.
pub mod connected_components;

/// In this module, everything concering the run of a transition system on a word is defined.
pub mod run;

/// This module defines traits for dealing with predecessors in a transition system.
pub mod predecessors;

/// Defines directed acyclic graphs (DAG)s and operations on them.
pub mod dag;

/// Encapsulates what is necessary for a type to be usable as a state index in a [`TransitionSystem`].
pub trait IndexType: Copy + std::hash::Hash + std::fmt::Debug + Eq + Ord + Display {}
impl<Idx: Copy + std::hash::Hash + std::fmt::Debug + Eq + Ord + Display> IndexType for Idx {}

/// Type for indices of states and edges.
pub type Idx = usize;

/// Helper trait for index types, which also allows conversion to a state or edge index.
pub trait Index {
    /// Turns self into an index of type [`Idx`].
    fn index(&self) -> Idx;

    /// Turns self into a state index.
    fn as_state_index(&self) -> StateIndex {
        StateIndex::new(self.index())
    }
}

impl Index for Idx {
    fn index(&self) -> Idx {
        *self
    }
}

/// Wrapper type for indices of states in a transition system.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct StateIndex(Idx);

impl StateIndex {
    /// Creates a new state index.
    pub fn new(index: Idx) -> Self {
        Self(index)
    }
}

impl Deref for StateIndex {
    type Target = Idx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Index for StateIndex {
    fn index(&self) -> Idx {
        self.0
    }
}

impl Display for StateIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.index())
    }
}

/// Implementors of this trait have a color, which can be obtained.
#[autoimpl(for<T: trait + ?Sized> &T, &mut T)]
pub trait HasColor {
    /// The color type of the implementor.
    type Color: Color;
    /// Returns a reference to the color of the implementor.
    fn color(&self) -> &Self::Color;
}

/// Implementors of this trait have a color, which can be obtained and set.
#[autoimpl(for<T: trait + ?Sized> &mut T)]
pub trait HasColorMut: HasColor {
    /// Sets the color of the implementor to the given color.
    fn set_color(&mut self, color: Self::Color);
}

/// A reference to a state in a transition system. This stores the index of the state and a
/// reference to the color of the state.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct StateReference<'a, Q> {
    /// The [`StateIndex`] of the state that is referenced.
    pub index: StateIndex,
    /// A reference to the color of the state.
    pub color: &'a Q,
}

impl<'a, Q: Color> HasColor for StateReference<'a, Q> {
    type Color = Q;
    fn color(&self) -> &Q {
        self.color
    }
}

impl<'a, Q> StateReference<'a, Q> {
    /// Creates a new state reference.
    pub fn new(index: StateIndex, color: &'a Q) -> Self {
        Self { index, color }
    }
}

/// Type alias for extracting the state color in a [`TransitionSystem`].
pub type StateColor<X> = <X as TransitionSystem>::StateColor;
/// Type alias for extracting the edge color in a [`TransitionSystem`].
pub type EdgeColor<X> = <X as TransitionSystem>::EdgeColor;

/// Abstracts possessing a set of states. Note, that implementors of this trait must
/// be able to iterate over the set of states.
#[autoimpl(for<T: trait + ?Sized> &T, &mut T)]
pub trait HasStates: TransitionSystem + Sized {
    /// The type of the states.
    type State<'this>: HasColor<Color = StateColor<Self>>
    where
        Self: 'this;

    /// The type of the iterator over the states.
    type StatesIter<'this>: Iterator<Item = (&'this Self::StateIndex, Self::State<'this>)>
    where
        Self: 'this;

    /// Returns a reference to the state with the given index, if it exists and `None` otherwise.
    fn state(&self, index: Self::StateIndex) -> Option<Self::State<'_>>;

    /// Returns an iterator over the states of the implementor.
    fn states_iter(&self) -> Self::StatesIter<'_>;

    /// Returns the number of states.
    fn hs_size(&self) -> usize {
        self.states_iter().count()
    }
}

/// Auxiliary type alias that allows easier access to the type of state index iterator
/// returned for transition systems that have a finite state space.
pub type FiniteStatesIterType<'a, This> = <This as HasFiniteStates<'a>>::StateIndicesIter;

/// Helper trait that must be implemented for every possible lifetime `'a`, in order
/// for a type to implement [`FiniteState`]. This is a (arguably hacky) solution to
/// deal with lifetime issues when iterating over the state indices of a transition
/// system. Especially interesting in the case of a [`DirectProduct`] for example.
pub trait HasFiniteStates<'a, Outlives = &'a Self>: TransitionSystem {
    /// Type of the iterator over the state indices of `Self`.
    type StateIndicesIter: Iterator<Item = Self::StateIndex>;
}

impl<'a, 'b, HFS: HasFiniteStates<'a>> HasFiniteStates<'a> for &'b HFS {
    type StateIndicesIter = <HFS as HasFiniteStates<'a>>::StateIndicesIter;
}

/// Implementors of this trait have a finite number of states and allow iteration over the
/// set of all state indices.
pub trait FiniteState: Sized + for<'a> HasFiniteStates<'a> {
    /// Returns an iterator over the state indices in `self`.
    fn state_indices(&self) -> FiniteStatesIterType<'_, Self>;

    /// Gives the size of `self`.
    fn size(&self) -> usize {
        self.state_indices().count()
    }

    /// Returns true if and only if the given state `index` exists.
    fn contains_state_index(&self, index: Self::StateIndex) -> bool {
        self.state_indices().contains(&index)
    }

    /// Tries to find the index of a state with the given `color`. Note that this uses `find` and thus
    /// returns the first such state that is found. There is no guarantee on the order in which the states
    /// are visited such that if more than one state with the given `color` exists, subsequent calls to
    /// this method may return different indices.
    fn find_by_color(&self, color: &StateColor<Self>) -> Option<Self::StateIndex> {
        self.state_indices()
            .find(|index| self.state_color(*index).as_ref() == Some(color))
    }

    /// Returns true if and only if a state with the given `color` exists.
    fn contains_state_color(&self, color: &StateColor<Self>) -> bool {
        self.find_by_color(color).is_some()
    }
}

impl<'a, FS: FiniteState> FiniteState for &'a FS {
    fn state_indices(&self) -> FiniteStatesIterType<'_, Self> {
        FS::state_indices(self)
    }
}

/// Abstracts possessing a set of states, which can be mutated. Note, that implementors of this
/// trait must be able to iterate over the set of states.
#[autoimpl(for<T: trait + ?Sized> &mut T)]

pub trait HasMutableStates: HasStates {
    /// The type of the mutable iterator over the states.
    type StateMut<'this>: HasColorMut<Color = StateColor<Self>>
    where
        Self: 'this;

    /// Returns an iterator over mutable references to the states of the implementor.
    fn state_mut(&mut self, index: Self::StateIndex) -> Option<Self::StateMut<'_>>;
}

/// Implementors of this trait have a distinguished (initial) state.
#[autoimpl(for<T: trait> &T, &mut T)]
pub trait Pointed: TransitionSystem {
    /// Returns the index of the initial state.
    fn initial(&self) -> Self::StateIndex;

    /// Returns the color of the initial state.
    fn initial_color(&self) -> Self::StateColor {
        self.state_color(self.initial())
            .expect("Initial state must exist and be colored!")
    }
}

/// This module deals with transforming a transition system (or similar) into a representation in the dot (graphviz) format.
pub mod dot;
pub use dot::ToDot;

mod quotient;
pub use quotient::Quotient;

use self::transition_system::IsTransition;

/// A congruence is a [`TransitionSystem`], which additionally has a distinguished initial state. On top
/// of that, a congruence does not have any coloring on either states or symbols. This
/// functionality is abstracted in [`Pointed`]. This trait is automatically implemented.
pub trait Congruence: TransitionSystem + Pointed {
    /// Creates a new instance of a [`RightCongruence`] from the transition structure of `self`. Returns
    /// the created congruence together with a [`Map`] from old/original state indices to indices of the
    /// created congruence.
    fn build_right_congruence(
        &self,
    ) -> (
        RightCongruence<Self::Alphabet>,
        Map<Self::StateIndex, usize>,
    )
    where
        Self: FiniteState,
    {
        let mut cong = RightCongruence::new_for_alphabet(self.alphabet().clone());
        let mut map = Map::default();

        for state in self.state_indices() {
            if self.initial() == state {
                map.insert(state, cong.initial());
                continue;
            }
            map.insert(state, cong.add_state(Class::epsilon()));
        }

        for state in self.state_indices() {
            if let Some(it) = self.edges_from(state) {
                for edge in it {
                    let target = edge.target();
                    let target_class = map.get(&target).unwrap();
                    let _color = edge.color().clone();
                    let _target_class = cong.add_edge(
                        *map.get(&state).unwrap(),
                        edge.expression().clone(),
                        *target_class,
                        (),
                    );
                }
            }
        }

        cong.recompute_labels();

        (cong, map)
    }
}
impl<Sim: TransitionSystem + Pointed> Congruence for Sim {}
