use std::collections::BTreeMap;

use crate::{
    alphabet::{Alphabet, HasAlphabet},
    Color,
};

use super::{
    ColorPosition, Edge, EdgeColor, EdgeIndex, EdgeIndicesFrom, EdgesFrom, HasMutableStates,
    HasStates, Index, IndexType, OnEdges, OnStates, Sproutable, State, StateColor, StateIndex,
    Successor, Transition,
};
/// An implementation of a transition system with states of type `Q` and colors of type `C`. It stores
/// the states and edges in a vector, which allows for fast access and iteration. The states and edges
/// are indexed by their position in the respective vector.
#[derive(Clone, Debug)]
pub struct IndexTS<A: Alphabet, C: Color, Pos: ColorPosition, Idx = usize> {
    alphabet: A,
    states: BTreeMap<Idx, State<Pos::StateColor<C>>>,
    edges: Vec<Edge<A::Expression, Pos::EdgeColor<C>, Idx>>,
}

pub type MealyTS<A, C, Idx = usize> = IndexTS<A, C, OnEdges, Idx>;
pub type MooreTS<A, C, Idx = usize> = IndexTS<A, C, OnStates, Idx>;

impl<A: Alphabet, Idx, C: Color, Position: ColorPosition> IndexTS<A, C, Position, Idx> {
    /// Creates a new transition system with the given alphabet.
    pub fn new(alphabet: A) -> Self {
        Self {
            alphabet,
            states: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    /// Gets a mutable reference to the alphabet of the transition system.
    pub fn alphabet(&self) -> &A {
        &self.alphabet
    }
}

impl<A: Alphabet, Idx: IndexType, Pos: ColorPosition, C: Color> IndexTS<A, C, Pos, Idx> {
    /// Returns an iterator over the [`EdgeIndex`]es of the edges leaving the given state.
    pub fn edge_indices_from(
        &self,
        source: Idx,
    ) -> EdgeIndicesFrom<'_, A::Expression, EdgeColor<Self>, Idx> {
        EdgeIndicesFrom::new(&self.edges, self.state(source).and_then(|s| s.first_edge()))
    }

    /// Returns an iterator over references to the edges leaving the given state.
    pub fn edges_from(&self, source: Idx) -> EdgesFrom<'_, A::Expression, EdgeColor<Self>, Idx> {
        EdgesFrom::new(&self.edges, self.state(source).and_then(|s| s.first_edge()))
    }

    /// Checks whether the state exists.
    pub fn contains_state<I: Into<Idx>>(&self, index: I) -> bool {
        self.states.contains_key(&index.into())
    }

    fn last_edge_from(&self, source: Idx) -> Option<EdgeIndex> {
        self.edge_indices_from(source).last()
    }

    fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<A::Expression, EdgeColor<Self>, Idx>> {
        self.edges.get(index.index())
    }

    fn get_edge_mut(
        &mut self,
        index: EdgeIndex,
    ) -> Option<&mut Edge<A::Expression, EdgeColor<Self>, Idx>> {
        self.edges.get_mut(index.index())
    }
}

impl<A: Alphabet, Pos: ColorPosition, C: Color> Sproutable for IndexTS<A, C, Pos, usize> {
    /// Adds a state with given `color` to the transition system, returning the [Idx] of
    /// the new state.
    fn add_state(&mut self, color: StateColor<Self>) -> usize {
        let id = self.states.len();
        let state = State::new(color);
        self.states.insert(id, state);
        id
    }

    /// Adds an edge from `source` to `target` with the given `trigger` and `color`, returning the
    /// [`EdgeIndex`] of the new edge. This method panics if `source` or `target` do not exist in
    /// the graph.
    fn add_edge<X, Y>(
        &mut self,
        from: X,
        on: <Self::Alphabet as Alphabet>::Expression,
        to: Y,
        color: EdgeColor<Self>,
    ) -> EdgeIndex
    where
        X: Into<usize>,
        Y: Into<usize>,
    {
        let source = from.into();
        let target = to.into();
        assert!(
            self.contains_state(source) && self.contains_state(target),
            "Source or target vertex does not exist in the graph."
        );
        let new_edge_id = self.edges.len().as_edge_index();

        let edge: Edge<<A as Alphabet>::Expression, EdgeColor<Self>, _> =
            if let Some(last_edge_id) = self.last_edge_from(source) {
                self.get_edge_mut(last_edge_id)
                    .unwrap()
                    .set_next(new_edge_id);
                Edge::new(source, target, color, on).with_prev(last_edge_id)
            } else {
                self.state_mut(source).unwrap().set_first_edge(new_edge_id);
                Edge::new(source, target, color, on)
            };
        self.edges.push(edge);
        new_edge_id
    }
}

impl<A: Alphabet, Idx: IndexType, Pos: ColorPosition, C: Color> Successor
    for IndexTS<A, C, Pos, Idx>
{
    type Position = Pos;
    type Color = C;
    type StateIndex = Idx;
    fn successor(
        &self,
        state: Idx,
        symbol: A::Symbol,
    ) -> Option<Transition<Idx, A::Symbol, EdgeColor<Self>>> {
        self.edges_from(state)
            .find(|e| self.alphabet().matches(e.trigger(), symbol))
            .map(|e| Transition::new(state, symbol, e.target(), e.color().clone()))
    }

    fn state_color(&self, index: Idx) -> StateColor<Self> {
        self.state(index)
            .map(|s| s.color().clone())
            .expect("cannot be called if state does not exist!")
    }
}

impl<A: Alphabet, Idx: IndexType, Pos: ColorPosition, C: Color> HasStates
    for IndexTS<A, C, Pos, Idx>
{
    type State<'this> = &'this State<Pos::StateColor<C>> where Self: 'this;

    type StatesIter<'this> = std::collections::btree_map::Iter<'this, Idx, State<Pos::StateColor<C>>>
    where
        Self: 'this;

    fn states_iter(&self) -> Self::StatesIter<'_> {
        self.states.iter()
    }
    fn state(&self, index: Idx) -> Option<Self::State<'_>> {
        self.states.get(&index)
    }
}

impl<A: Alphabet, Idx: IndexType, Pos: ColorPosition, C: Color> HasMutableStates
    for IndexTS<A, C, Pos, Idx>
{
    type StateMut<'this> = &'this mut State<Pos::StateColor<C>> where Self: 'this;

    fn state_mut(&mut self, index: Idx) -> Option<Self::StateMut<'_>> {
        self.states.get_mut(&index)
    }
}

impl<A: Alphabet, Idx: IndexType, Pos: ColorPosition, C: Color> HasAlphabet
    for IndexTS<A, C, Pos, Idx>
{
    type Alphabet = A;
    fn alphabet(&self) -> &Self::Alphabet {
        &self.alphabet
    }
}

trait Increment {
    fn increment(&mut self);
}

impl<X> Increment for X
where
    X: std::ops::AddAssign<u8>,
{
    fn increment(&mut self) {
        *self += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alphabet,
        ts::{index_ts::MealyTS, Sproutable, Successor, Transition},
    };

    use super::IndexTS;

    #[test]
    fn build_ts() {
        let mut ts = MealyTS::new(alphabet::Simple::from_iter(['a', 'b']));
        let s0 = ts.add_state(());
        let s1 = ts.add_state(());
        let _e0 = ts.add_edge(s0, 'a', s1, 0);
        let _e1 = ts.add_edge(s0, 'b', s0, 1);
        let _e2 = ts.add_edge(s1, 'a', s1, 0);
        let _e3 = ts.add_edge(s1, 'b', s0, 1);
        println!("{:?}", ts);
        assert!(ts.successor(s0, 'a').is_some());
        assert_eq!(ts.successor(s1, 'a'), Some(Transition::new(s1, 'a', s1, 0)));
        assert_eq!(ts.edges_from(s0).count(), 2);
    }
}
