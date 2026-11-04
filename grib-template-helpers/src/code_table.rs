/// Resolves a code using external information supplied by the caller.
///
/// Use `&()` for context-independent lookups.
pub trait CodeTable<C: ?Sized = ()> {
    /// Whether any entry in the entire table defines a unit.
    ///
    /// This includes all disciplines and parameter categories, independently of
    /// the current code or context. When true, `code_unit` may still return
    /// `None` for an individual lookup. When false, it always returns `None`.
    const HAS_UNIT: bool;

    /// Whether this code is explicitly defined as missing by the table.
    fn code_is_missing(&self, _context: &C) -> bool {
        false
    }

    /// Returns the description for the code number.
    fn code_description(&self, context: &C) -> Option<&'static str>;

    /// Returns the textual representation of the defined unit.
    fn code_unit(&self, context: &C) -> Option<&'static str>;
}
