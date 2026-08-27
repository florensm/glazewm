//! Type for parsing a fixed-order sequence of items, separated by a fixed
//! separator, from a `syn::parse::ParseStream`.

/// Trait for tuples where all items can be parsed from a
/// parse stream.
pub trait ParseableTuple
where
  Self: Sized,
{
  type FirstItem: syn::parse::Parse;

  /// Parses all items in the tuple `T` from the stream in the order they
  /// appear in the tuple, and parses `Sep` in between each item. Returns
  /// all parsed items in a tuple, or the first error to occur.
  ///
  /// Do not use this directly, use the [Ordered] type instead,
  fn parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self>
  where
    Sep: syn::parse::Parse;
}

macro_rules! get_first_item {
  ($first:tt, $($types:tt),+) => {
    $first
  };
}

macro_rules! impl_for_tuple {
  ($($types:tt),+ | $($numbers:tt),+) => {
    // Generic ensures that all types in the tuple implement `syn::parse::Parse`.
    impl<$($types),+> ParseableTuple for ($($types,)+) where $($types : syn::parse::Parse),+ {
      type FirstItem = get_first_item!($($types),+);

      fn parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self> where Sep: syn::parse::Parse {
        $(
          // Also check if the stream is empty to allow for missing trailing Optional items
          if $numbers != 0 && !stream.is_empty() {
            stream.parse::<Sep>()?;
          }

          #[allow(non_snake_case)]
          let $types = stream.parse::<$types>()?;
        )+

        // Pack the parsed items into a tuple
        Ok(($($types),+))
      }
    }
  };
}

// Implement `ParseableTuple` for 2-tuples -- the only tuple size [Ordered]
// is ever used with.
impl_for_tuple!(T, U | 0, 1);

/// Type wrapper to parse all items in tuple `T` in order, using
/// `Sep` as the separator between items.
///
/// # Example
/// Parse [syn::Ident] and [syn::LitStr] from the stream, which are
/// separated by a comma. E.g. `some_name, "some string"`. If the order is
/// reversed, it will fail to parse.
/// ```ignore
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
///   type T = (syn::Ident, syn::LitStr);
///
///   stream.parse::<Ordered<T, syn::Token![,]>>()?;
/// }
///
/// fn main() {
///   # use quote::quote;
///   let tokens = quote! { some_name, "some string" }.into();
///   let error = quote! { "some string", some_name }.into();
///
///   assert!(example(tokens).is_ok());
///   assert!(example(error).is_err());
/// }
/// ```
pub struct Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  pub items: T,
  sep: std::marker::PhantomData<Sep>,
}

impl<T, Sep> syn::parse::Parse for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let output = T::parse_tuple::<Sep>(input)? as T;
    Ok(Self {
      items: output,
      sep: std::marker::PhantomData,
    })
  }
}

/// Implement [Peekable] for [Ordered] if the first item in the
/// tuple is peekable, by forwarding the implementation to the first item.
impl<T, FirstItem, Sep> crate::common::peekable::Peekable
  for Ordered<T, Sep>
where
  T: ParseableTuple<FirstItem = FirstItem>,
  Sep: syn::parse::Parse,
  FirstItem: crate::common::peekable::Peekable,
{
  fn display() -> &'static str {
    FirstItem::display()
  }

  fn peek<S>(stream: S) -> bool
  where
    S: crate::common::peekable::PeekableStream,
  {
    FirstItem::peek(stream)
  }
}

impl<T, Sep> core::ops::Deref for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.items
  }
}

