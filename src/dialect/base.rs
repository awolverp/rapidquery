implement_pyclass! {
    /// A base class for all supported dialects
    ///
    /// DO NOT try to create an instance of this class (DO NOT call __new__ method).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    [skip_from_py_object, subclass] PyBaseDialect as "BaseDialect";
}
