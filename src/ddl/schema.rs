/// A `CREATE SCHEMA` statement.
#[rapidquery_macros::pyclass("rapidquery.ddl.CreateSchema", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyCreateSchema {
    pub schema_name: arcstr::ArcStr,
    pub if_not_exists: bool,
    pub owner_name: Option<arcstr::ArcStr>,
    pub comment: Option<arcstr::ArcStr>,
    pub location: Option<arcstr::ArcStr>,
}
