pub trait CodeTable {
    /// Returns the description for the code number.
    fn code_description(&self) -> Option<&'static str>;

    /// Returns the textual representation of the defined unit.
    fn code_unit(&self) -> Option<&'static str>;
}
