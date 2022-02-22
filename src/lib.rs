//! Escape values while they are being formatted
//!
//! When processing data, and particularly when forwarding data, in the form of
//! character sequences (strings, streams, ...), one needs to escape certain
//! characters or constructs which have some special meaning in the format. This
//! crate provides [Escaped], a wrapper implementing [Display] such that the
//! inner value is automatically escaped when formatted.
//!
//! The escaping logic can be customized via the [Escaper] trait, or by
//! supplying an `FnMut(char) -> Display + Clone`.
//!
//! Rather than importing [Escaped] directly, users are encouraged to import
//! [Escapable] instead. This convenience trait augments all [Sized] [Display]
//! types with functions wrapping the value in an [Escaped] (for unsized types,
//! a reference to the value may be used instead):
//!
//! ```
//! use rescue_blanket::Escapable;
//! println!("foo=\"{}\"", "bar=\"baz\"".escaped_with(char::escape_default));
//! ```
//!
//! # Why using `rescue_blanket`?
//!
//! There are a number of crates already for escaping strings, and there are
//! already [str::escape_default], [escape_debug](str::escape_debug) and
//! [escape_unicode](str::escape_unicode), so why yet another library?
//!
//! These functions, and the libraries I found, only work well if the values
//! you need to escape are already accessible as [str]. However, sometimes your
//! values are more complex, maybe recursive, and you may not want put escaping
//! logic inside their [Display] implementation. After all, the need for
//! escaping arises from the context the value is formatted in, not from the
//! value itself.
//!
//! You could always format complex values into some buffer (e.g. [String]) and
//! apply escaping on the result, but that requires the additional buffer and
//! you may want to avoid that. Depending on the [Escaper], the use of [Escaped]
//! does not involve any additional buffering.

use core::fmt::{self, Display};


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

    /// Wrap this value in an [Escaped] for escaped formatting
    ///
    /// The resulting [Escaped] will escape the value when being formatted via
    /// [Display] using the given [Escaper].
    fn escaped_with_default<E: Escaper + Default>(self) -> Escaped<Self, E> {
        Escaped::new_default(self)
    }

    /// Wrap this value in an [Escaped] for escaping with [char::escape_default]
    ///
    /// The resulting [Escaped] will escape the value when being formatted via
    /// [Display] using [char::escape_default] as [Escaper].
    ///
    /// # Examples
    ///
    /// ```
    /// use rescue_blanket::Escapable;
    /// let s = "foo=\"bar\"";
    /// // Compare against str::escape_default()
    /// assert_eq!(s.escaped_default().to_string(), s.escape_default().to_string());
    /// ```
    fn escaped_default(self) -> Escaped<Self, fn(char) -> core::char::EscapeDefault> {
        self.escaped_with(char::escape_default)
    }

    /// Wrap this value in an [Escaped] for escaping with [char::escape_debug]
    ///
    /// The resulting [Escaped] will escape the value when being formatted via
    /// [Display] using [char::escape_debug] as [Escaper].
    ///
    /// # Examples
    ///
    /// ```
    /// use rescue_blanket::Escapable;
    /// let s = "foo=\"bar\"";
    /// // Compare against str::escape_debug()
    /// assert_eq!(s.escaped_debug().to_string(), s.escape_debug().to_string());
    /// ```
    fn escaped_debug(self) -> Escaped<Self, fn(char) -> core::char::EscapeDebug> {
        self.escaped_with(char::escape_debug)
    }

    /// Wrap this value in an [Escaped] for escaping with [char::escape_unicode]
    ///
    /// The resulting [Escaped] will escape the value when being formatted via
    /// [Display] using [char::escape_unicode] as [Escaper].
    ///
    /// # Examples
    ///
    /// ```
    /// use rescue_blanket::Escapable;
    /// let s = "foo=\"bar\"";
    /// // Compare against str::escape_unicode()
    /// assert_eq!(s.escaped_unicode().to_string(), s.escape_unicode().to_string());
    /// ```
    fn escaped_unicode(self) -> Escaped<Self, fn(char) -> core::char::EscapeUnicode> {
        self.escaped_with(char::escape_unicode)
    }
}

impl<T: Display> Escapable for T {
    fn escaped_with<E: Escaper>(self, escaper: E) -> Escaped<Self, E> {
        Escaped::new(self, escaper)
    }
}

