//! SQL data types: interned names, serializers, and the composed [`DataType`].
//!
//! A [`DataTypeName`] is a canonical spelling plus a [`DataTypeGroup`]. The
//! group is the inference key for which [`Serializer`] [`DataType::from_name`]
//! constructs when none is passed. Groups are serializer-oriented: `NUMERIC`
//! and `DECIMAL` sit in [`DataTypeGroup::Float`] because they share a
//! serializer with IEEE floats.

mod name;
mod serialize;
mod serializer;

#[cfg(test)]
mod tests;

pub use name::{DataTypeGroup, DataTypeName};
pub use serializer::*;

use crate::native::sql::{PrepareResult, SqlWriter, WriteSql};

/// SQL type spelling plus the serializer used for Python values.
///
/// [`WriteSql`] emits the type name (CAST, column definitions). Literal
/// serialization of a Python object is [`Serializer::write_sql`], not this
/// type.
///
/// Clone of a [`Serializer::Custom`] value requires the GIL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataType {
    pub name: DataTypeName,
    pub serializer: Serializer,
}

impl DataType {
    #[inline]
    pub fn new(name: DataTypeName, serializer: Serializer) -> Self {
        Self { name, serializer }
    }

    /// Infer the serializer from [`DataTypeName::group`].
    ///
    /// Unrecognized and parameterized names use [`Serializer::Unspecified`].
    #[inline]
    pub fn from_name(name: impl Into<DataTypeName>) -> Self {
        let name = name.into();
        let serializer = Serializer::from_group(name.group());
        Self { name, serializer }
    }
}

impl WriteSql for DataType {
    #[inline]
    fn write_sql<W: SqlWriter>(
        &self,
        dialect: &crate::native::dialect::SqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        self.name.write_sql(dialect, out)
    }
}
