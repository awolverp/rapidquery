// use crate::common::PyQueryStatement;
// use crate::common::PyTableName;
// use crate::expression::PyFunc;

// pub enum SelectReference {
//     SubQuery(PySelectStatement, String),
//     FunctionCall(PyFunc, String),
//     TableName(PyTableName),
// }

// implement_pyclass! {
//     /// Represents a column expression with an optional alias in a SELECT statement.
//     ///
//     /// Used to specify both the expression to select and an optional alias name
//     /// for the result column.
//     ///
//     /// @signature (expr: object, /, alias: str | None = None, )
//     pub struct [] PySelectExpr as "SelectExpr" {

//     }
// }

// implement_state_pyclass! {
//     /// Builds SELECT SQL statements with a fluent interface.
//     ///
//     /// Provides a chainable API for constructing SELECT queries with support for:
//     /// - Column selection with expressions and aliases
//     /// - Table and subquery sources
//     /// - Filtering with WHERE and HAVING
//     /// - Joins (inner, left, right, full, cross, lateral)
//     /// - Grouping and aggregation
//     /// - Ordering and pagination
//     /// - Set operations (UNION, EXCEPT, INTERSECT)
//     /// - Row locking for transactions
//     /// - DISTINCT queries
//     pub struct [extends=PyQueryStatement] PySelectStatement(SelectStatementState) as "SelectStatement" {
//         // TODO: support from_values
//         references: Vec<SelectReference>,
//     }
// }
