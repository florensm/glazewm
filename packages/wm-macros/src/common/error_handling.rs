//! Utilities for simplifying error handling and diagnostics.

pub mod prelude {
  pub use super::{EmitError, ThenError, ToError};
}

/// Extends the `bool` type with a method that returns an error if the
/// value is `true`.
pub trait ThenError<E>
where
  Self: Sized,
{
  /// Returns `Err(error)` if the value is `true`, otherwise returns
  /// `Ok(self)`.
  ///
  /// # Example
  /// ```ignore
  /// # fn example(string: &str, string_span: syn::Span) -> syn::Result<()> {
  /// string.is_empty().then_error(string_span.error("Expected a non-empty string"))?;
  /// # }
  /// ```
  fn then_error(self, error: E) -> Result<Self, E>;
}

impl<E> ThenError<E> for bool {
  fn then_error(self, error: E) -> Result<Self, E> {
    if self { Err(error) } else { Ok(self) }
  }
}

/// Extension trait for any type that can be tokenized to create a
/// [syn::Error] at its location.
pub trait ToError {
  /// Create a [syn::Error] at the span of this object with the given
  /// message.
  ///
  /// # Example
  /// ```ignore
  /// # fn example(ident: syn::Ident) -> syn::Result<()> {
  /// return Err(ident.error("Didn't expect an identifier here"));
  /// # }
  /// ```
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error;
}

impl<T> ToError for T
where
  T: quote::ToTokens,
{
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error {
    syn::Error::new_spanned(self, message)
  }
}

pub trait EmitError {
  /// Directly emits a warning message at the span of this object.
  fn emit_warning<D: Into<String>>(&self, message: D);
}

impl<T> EmitError for T
where
  T: syn::spanned::Spanned,
{
  fn emit_warning<D: Into<String>>(&self, message: D) {
    self.span().unwrap().warning(message).emit();
  }
}
