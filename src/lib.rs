
use std::fmt::{self, Display};


/// Character-wise processor implementing some escaping logic
///
/// Types implementing this trait define how a string or value is escaped, based
/// on individual `char`s. An impls' [process](Escaper::process) function will
/// receive one character and produce an appropriate [Output](Escaper::Output)
/// implementing [Display].
///
/// In simple cases, the output will display as either the input character or,
/// if the character needs to be escaped, an appropriate escape sequence. In
/// more complex cases, the escaping logic may end up being a state machine of
/// some kind driven by input `char`s. In order to support such use-cases,
/// [process](Escaper::process) takes a mutable reference of `self`, allowing it
/// to carry state across invocations.
///
/// # Note
///
/// An `Escaper` needs to implement [Clone]. However, escaping of a single
/// string or value is to be performed on the same instance. Clones do not
/// expected to share any state.
///
/// # Note
///
/// A blanket implementation for `FnMut(char) -> impl Display + Clone` is
/// provided for users' convenience.
pub trait Escaper: Clone {
    /// Partial output after escaping
    ///
    /// This type represents the output of processing a single input `char`.
    type Output: Display;

    /// Process a single input character
    ///
    /// This function processes a single input `char` and produces as a result
    /// an appropriate [Output](Escaper::Output). The concatenation of the
    /// results of [ToString::to_string] via [Display] for each
    /// [Output](Escaper::Output) results in a correctly escaped `String`.
    fn process(&mut self, input: char) -> Self::Output;
}

impl<F: FnMut(char) -> O + Clone, O: Display> Escaper for F {
    type Output = O;

    fn process(&mut self, input: char) -> Self::Output {
        self(input)
    }
}

