/// Resolves a code using external information supplied by the caller.
///
/// Use `&()` for context-independent lookups.
pub trait CodeTable<C: ?Sized = ()> {
    /// Returns the description for the code number.
    fn code_description(&self, context: &C) -> Option<&'static str>;

    /// Returns the textual representation of the defined unit.
    fn code_unit(&self, context: &C) -> Option<&'static str>;
}
