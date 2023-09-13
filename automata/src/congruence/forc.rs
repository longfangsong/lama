use crate::{ts::transition_system::Indexes, Alphabet, Class, Color, Map, RightCongruence};

/// A family of right congruences (FORC) consists of a *leading* right congruence and for each
/// class of this congruence a *progress* right congruence.
#[derive(Clone, PartialEq, Eq)]
pub struct FORC<A: Alphabet, Q: Color = (), C: Color = ()> {
    pub(crate) leading: RightCongruence<A>,
    pub(crate) progress: Map<usize, RightCongruence<A, Q, C>>,
}

impl<A: Alphabet, Q: Color, C: Color> FORC<A, Q, C> {
    /// Creates a new FORC with the given leading congruence and progress congruences.
    pub fn new(
        leading: RightCongruence<A>,
        progress: Map<usize, RightCongruence<A, Q, C>>,
    ) -> Self {
        Self { leading, progress }
    }

    pub fn leading(&self) -> &RightCongruence<A> {
        &self.leading
    }

    /// Insert a new progress congruence for the given class.
    pub fn insert<X>(&mut self, class: X, congruence: RightCongruence<A, Q, C>)
    where
        X: Indexes<RightCongruence<A>>,
    {
        let idx = class
            .to_index(self.leading())
            .expect("Cannot add prc for class that does not exist!");
        self.progress.insert(idx, congruence);
    }

    /// Tries to obtain a reference to the progress right congruence for the given `class`.
    pub fn prc<X>(&self, class: X) -> Option<&RightCongruence<A, Q, C>>
    where
        X: Indexes<RightCongruence<A>>,
    {
        let idx = class.to_index(self.leading())?;
        self.progress.get(&idx)
    }

    /// Creates a new FORC from the given leading congruence and progress congruences.
    pub fn from_iter<I: IntoIterator<Item = (usize, RightCongruence<A, Q, C>)>>(
        leading: RightCongruence<A>,
        progress: I,
    ) -> Self {
        Self {
            leading,
            progress: progress.into_iter().collect(),
        }
    }
}
