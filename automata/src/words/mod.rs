use std::fmt::Debug;

use crate::{
    congruence::CongruenceTransition, ts::IntoTransitions, Boundedness, Class, FiniteKind,
    RightCongruence, Set, Successor, Symbol,
};
mod append;
pub use append::Append;

mod prepend;
use impl_tools::autoimpl;
pub use prepend::Prepend;

mod finite;
mod infinite;
mod subword;

pub use finite::Str;
pub use infinite::{PeriodicWord, UltimatelyPeriodicWord};
pub use subword::Subword;
use tracing::trace;

/// Abstracts a word over some given alphabet. The type parameter `S` is the alphabet, and `Kind` is a marker type which indicates whether the word is finite or infinite.
#[autoimpl(for<T: trait> &T, &mut T)]
pub trait Word: Debug + Eq + std::hash::Hash {
    /// The type of the symbols making up the word.
    type S: Symbol;

    /// Indicates whether the word is finite or infinite.
    type Kind: Boundedness;

    /// Returns the symbol at the given index, or `None` if the index is out of bounds.
    fn nth(&self, index: usize) -> Option<Self::S>;
}

/// Alias to extract the kind of the word.
pub type WordKind<W> = <W as Word>::Kind;

/// A trait which indicates that a word is finite.
#[autoimpl(for<T: trait> &T, &mut T)]
pub trait IsFinite: Word {
    /// Returns the length of the word.
    fn length(&self) -> usize;
}

/// Marker trait for infinite words, assumes the implementor is finitely representable as an ultimately periodic word, i.e. a base word followed by an infinitely looping non-empty word.
pub trait IsInfinite: Word {
    /// Returns the length of the base word.
    fn base_length(&self) -> usize;
    /// Returns the length of the recurring word.
    fn recur_length(&self) -> usize;
}

impl<F: IsInfinite> IsInfinite for &F {
    fn base_length(&self) -> usize {
        IsInfinite::base_length(*self)
    }

    fn recur_length(&self) -> usize {
        IsInfinite::recur_length(*self)
    }
}

/// A trait which allows iterating over the symbols of a word. For an infinite word, this is an infinite iterator.
pub trait SymbolIterable: Word {
    /// The iterator type.
    type Iter: Iterator<Item = Self::S>;

    /// Returns an iterator over the symbols of the word.
    fn iter(&self) -> Self::Iter;
}

impl Word for String {
    type Kind = FiniteKind;
    type S = char;

    fn nth(&self, index: usize) -> Option<Self::S> {
        self.chars().nth(index)
    }
}

impl IsFinite for String {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Word for &str {
    type Kind = FiniteKind;
    type S = char;

    fn nth(&self, index: usize) -> Option<Self::S> {
        self.chars().nth(index)
    }
}

impl IsFinite for &str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<S: Symbol> IsFinite for Vec<S> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<S: Symbol> Word for Vec<S> {
    type Kind = FiniteKind;
    type S = S;

    fn nth(&self, index: usize) -> Option<Self::S> {
        self.get(index).cloned()
    }
}

/// Used to extract the transitions from a word viewed as a transition system.
/// Is an iterator that outputs the transitions as follows:
///
/// For a finite word w = abaab, it would emit transitions
///     ε -a-> a, a -b-> ab, ab -a-> aba, ...
///
/// Infinite words like w = uv^ω will produce transitions that mimick a finite path
/// on the symbols of u, to which a loop on the symbols of v is attached
#[derive(Debug, Clone)]
pub struct WordTransitions<W: Subword> {
    word: W,
    pos: usize,
}

impl<S: Symbol> Iterator for WordTransitions<&UltimatelyPeriodicWord<S>> {
    type Item = CongruenceTransition<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let loop_back_point = self.word.base_length() + self.word.recur_length();

        trace!(
            "Pos is {}/{}, base {} and recur {}",
            self.pos,
            loop_back_point,
            self.word.base_length(),
            self.word.recur_length()
        );
        let ret = match self.pos.cmp(&loop_back_point) {
            std::cmp::Ordering::Less => Some((
                self.word.prefix(self.pos).into(),
                self.word
                    .nth(self.pos)
                    .expect("Was checked via base length"),
                self.word.prefix(self.pos + 1).into(),
            )),
            std::cmp::Ordering::Equal => Some((
                self.word.prefix(self.pos).into(),
                self.word
                    .nth(self.pos)
                    .expect("Should also be covered by length!"),
                self.word.prefix(self.word.base_length() + 1).into(),
            )),
            std::cmp::Ordering::Greater => None,
        };
        self.pos += 1;
        ret
    }
}

impl<W: Subword> WordTransitions<W> {
    /// Creates a new [`WordTransitions`] object.
    pub fn new(word: W) -> Self {
        Self { word, pos: 0 }
    }
}

impl<W> Iterator for WordTransitions<W>
where
    W: Subword + IsFinite,
    W::PrefixType: Into<Class<W::S>>,
{
    type Item = CongruenceTransition<W::S>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = if self.pos < self.word.length() {
            trace!("Pos is {}", self.pos);
            Some((
                self.word.prefix(self.pos).into(),
                self.word
                    .nth(self.pos)
                    .expect("Was checked via base length"),
                self.word.prefix(self.pos + 1).into(),
            ))
        } else {
            None
        };
        self.pos += 1;
        ret
    }
}

/// A macro for constructing an ultimately periodic word from string(s).
#[macro_export]
macro_rules! upw {
    ($cyc:expr) => {
        $crate::words::UltimatelyPeriodicWord::from($crate::words::PeriodicWord::from($cyc))
    };
    ($base:expr, $cyc:expr) => {
        $crate::words::UltimatelyPeriodicWord::from((
            $crate::words::Str::from($base),
            $crate::words::PeriodicWord::from($cyc),
        ))
    };
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    #[test]
    fn symbol_iterability() {
        let word = Str::<usize>::from(vec![1, 3, 3, 7]);
        let mut iter = word.iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), None);

        let word = UltimatelyPeriodicWord(Str::empty(), PeriodicWord::from(vec![1, 3, 3, 7]));
        let mut iter = word.iter();
        assert_eq!(iter.next(), Some(1usize));
    }

    #[test]
    #[traced_test]
    fn word_as_ts() {
        let word = upw!("aa", "bb");
        let ts = word.into_ts();
        println!("{}", ts);
    }
}
