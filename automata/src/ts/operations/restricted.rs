use crate::{
    alphabet::HasAlphabet,
    ts::{
        predecessors::{IsPreTransition, PredecessorIterable},
        transition_system::IsTransition,
        FiniteState, FiniteStatesIterType, HasFiniteStates,
    },
    Pointed, TransitionSystem,
};

/// Restricts a transition system to a subset of its state indices, which is defined by a filter
/// function.

#[derive(Debug, Clone)]
pub struct RestrictByStateIndex<Ts: TransitionSystem, F> {
    ts: Ts,
    filter: F,
}

/// Iterator over the state indices of a transition system that are restricted by a filter function.
pub struct RestrictByStateIndexIter<'a, Ts: TransitionSystem + HasFiniteStates<'a>, F> {
    filter: &'a F,
    it: FiniteStatesIterType<'a, Ts>,
}

impl<'a, Ts: TransitionSystem + HasFiniteStates<'a>, F: Fn(Ts::StateIndex) -> bool> Iterator
    for RestrictByStateIndexIter<'a, Ts, F>
{
    type Item = Ts::StateIndex;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.find(|idx| (self.filter)(*idx))
    }
}

impl<'a, Ts: TransitionSystem + HasFiniteStates<'a>, F> RestrictByStateIndexIter<'a, Ts, F> {
    /// Creates a new iterator over the state indices of a transition system that are restricted by a
    /// filter function.
    pub fn new(filter: &'a F, it: FiniteStatesIterType<'a, Ts>) -> Self {
        Self { filter, it }
    }
}

impl<'a, Ts, F> HasFiniteStates<'a> for RestrictByStateIndex<Ts, F>
where
    Ts: HasFiniteStates<'a>,
    F: Fn(Ts::StateIndex) -> bool,
{
    type StateIndicesIter = RestrictByStateIndexIter<'a, Ts, F>;
}

impl<Ts, F> FiniteState for RestrictByStateIndex<Ts, F>
where
    Ts: FiniteState,
    F: Fn(Ts::StateIndex) -> bool,
{
    fn state_indices(&self) -> crate::ts::sealed::FiniteStatesIterType<'_, Self> {
        RestrictByStateIndexIter::new(&self.filter, self.ts.state_indices())
    }
}

impl<Ts: TransitionSystem + Pointed, F> Pointed for RestrictByStateIndex<Ts, F>
where
    F: Fn(Ts::StateIndex) -> bool,
{
    fn initial(&self) -> Self::StateIndex {
        let initial = self.ts.initial();
        assert!((self.filter)(initial), "initial state is filtered out");
        initial
    }
}

impl<Ts, F> HasAlphabet for RestrictByStateIndex<Ts, F>
where
    Ts: TransitionSystem,
{
    type Alphabet = Ts::Alphabet;
    fn alphabet(&self) -> &Self::Alphabet {
        self.ts.alphabet()
    }
}

#[allow(missing_docs)]
impl<Ts: TransitionSystem, F> RestrictByStateIndex<Ts, F> {
    pub fn new(ts: Ts, filter: F) -> Self {
        Self { ts, filter }
    }

    pub fn filter(&self) -> &F {
        &self.filter
    }

    pub fn ts(&self) -> &Ts {
        &self.ts
    }
}

/// Iterator over the edges of a transition system that are restricted by a filter function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictedEdgesFromIter<'a, Ts: TransitionSystem + 'a, F> {
    filter: &'a F,
    it: Ts::EdgesFromIter<'a>,
}

#[allow(missing_docs)]
impl<'a, Ts: TransitionSystem + 'a, F> RestrictedEdgesFromIter<'a, Ts, F> {
    pub fn new(it: Ts::EdgesFromIter<'a>, filter: &'a F) -> Self {
        Self { filter, it }
    }
}

impl<'a, Ts: TransitionSystem + 'a, F> Iterator for RestrictedEdgesFromIter<'a, Ts, F>
where
    F: Fn(Ts::StateIndex) -> bool,
{
    type Item = Ts::TransitionRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.by_ref().find(|edge| (self.filter)(edge.target()))
    }
}

/// Iterator over the predecessors in a transition system that are restricted by a filter function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictedEdgesToIter<'a, Ts: PredecessorIterable + 'a, F> {
    filter: &'a F,
    it: Ts::EdgesToIter<'a>,
}

impl<'a, Ts: PredecessorIterable + 'a, F> Iterator for RestrictedEdgesToIter<'a, Ts, F>
where
    F: Fn(Ts::StateIndex) -> bool,
{
    type Item = Ts::PreTransitionRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.by_ref().find(|edge| (self.filter)(edge.source()))
    }
}

#[allow(missing_docs)]
impl<'a, Ts: PredecessorIterable + 'a, F> RestrictedEdgesToIter<'a, Ts, F> {
    pub fn new(it: Ts::EdgesToIter<'a>, filter: &'a F) -> Self {
        Self { filter, it }
    }
}

#[cfg(test)]
mod tests {
    use crate::{simple, ts::Sproutable, Acceptor, Pointed, TransitionSystem, DFA};

    #[test]
    fn restrict_ts_by_state_index() {
        let mut dfa = DFA::new(simple! {'a', 'b'});
        let q0 = dfa.initial();
        let q1 = dfa.add_state(false);
        let q2 = dfa.add_state(true);

        dfa.add_edge(q0, 'a', q1, ());
        dfa.add_edge(q0, 'b', q0, ());
        dfa.add_edge(q1, 'a', q2, ());
        dfa.add_edge(q1, 'b', q1, ());
        dfa.add_edge(q2, 'a', q0, ());
        dfa.add_edge(q2, 'b', q2, ());
        assert!(dfa.accepts("aa"));

        let restricted = dfa.restrict_state_indices(|idx| idx != q2);
        assert!(!restricted.accepts("aa"));
    }
}
