
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


/// Wrapper for escaping items during formatting
///
/// This type wraps an item implementing [Display] together with an [Escaper].
/// When displayed via its own implementation of [Display], the encapsulated
/// item will be escaped via the [Escaper] during the formatting process.
///
/// # Note
///
/// Users of the library will usually prefer importing and using [Escapable]
/// over using this type directly. An exception may be the construction of
/// interfaces enforcing some sort of escaping for inputs.
///
/// # Examples
///
/// ```
/// let escaped = rescue_blanket::Escaped::new("foo=\"bar\"", char::escape_default);
/// assert_eq!(escaped.to_string(), "foo=\\\"bar\\\"");
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Escaped<I: fmt::Display, E: Escaper> {
    item: I,
    escaper: E,
}

impl<I: fmt::Display, E: Escaper> Escaped<I, E> {
    /// Create a new wrapper for the given item with an [Escaper]
    pub fn new(item: I, escaper: E) -> Self {
        Self {item, escaper}
    }

    /// Create a new wrapper for the given item with a default [Escaper]
    pub fn new_default(item: I) -> Self where E: Default {
        Self {item, escaper: Default::default()}
    }
}

impl<I: fmt::Display, E: Escaper + Default> From<I> for Escaped<I, E> {
    fn from(item: I) -> Self {
        Self::new_default(item)
    }
}

impl<I: fmt::Display, E: Escaper> Display for Escaped<I, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        let mut out = WriteProxy::new(f, self.escaper.clone());
        write!(out, "{}", self.item)
    }
}


/// Escaping [fmt::Write] implementation
struct WriteProxy<'a, 'b, E: Escaper> {
    formatter: &'a mut fmt::Formatter<'b>,
    escaper: E,
}

impl<'a, 'b, E: Escaper> WriteProxy<'a, 'b, E> {
    /// Create a new proxy
    fn new(formatter: &'a mut fmt::Formatter<'b>, escaper: E) -> Self {
        Self {formatter, escaper}
    }
}

impl<E: Escaper> fmt::Write for WriteProxy<'_, '_, E> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().try_for_each(|c| self.write_char(c))
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.escaper.process(c).fmt(self.formatter)
    }
}


/// Convenience trait for escaping items
///
/// This trait augments types implementing [Display] with functions for wrapping
/// them in instances of [Escaped], which will escape the value when being
/// formatted.
///
/// # Examples
///
/// ```
/// use rescue_blanket::Escapable;
/// assert_eq!("foo=\"bar\"".escaped_with(char::escape_default).to_string(), "foo=\\\"bar\\\"");
/// ```
pub trait Escapable: Display + Sized {
    /// Wrap this value in an [Escaped] for escaped formatting
    ///
    /// The resulting [Escaped] will escape the value when being formatted via
    /// [Display] using the given [Escaper].
    fn escaped_with<E: Escaper>(self, escaper: E) -> Escaped<Self, E>;
}

impl<T: Display> Escapable for T {
    fn escaped_with<E: Escaper>(self, escaper: E) -> Escaped<Self, E> {
        Escaped::new(self, escaper)
    }
}

