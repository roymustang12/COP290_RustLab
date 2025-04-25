use crate::cellsp::CellReference;

/// Represents an expression in the spreadsheet.
///
/// This enum is used to define various types of expressions that can be evaluated
/// in the spreadsheet, such as numbers, cell references, binary operations, functions, and ranges.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A constant integer value.
    Number(i32),
    /// A reference to a specific cell in the spreadsheet.
    Cell(CellReference),
    /// A binary operation between two expressions.
    ///
    /// # Fields
    /// * `Box<Expr>` - The left-hand side of the binary operation.
    /// * `char` - The operator (e.g., `+`, `-`, `*`, `/`).
    /// * `Box<Expr>` - The right-hand side of the binary operation.
    BinaryOp(Box<Expr>, char, Box<Expr>),
    /// A function applied to a list of expressions.
    ///
    /// # Fields
    /// * `String` - The name of the function (e.g., `SUM`, `AVG`).
    /// * `Vec<Expr>` - The arguments to the function.
    Function(String, Vec<Expr>),
    /// A range of cells in the spreadsheet.
    ///
    /// # Fields
    /// * `CellReference` - The starting cell of the range.
    /// * `CellReference` - The ending cell of the range.
    Range(CellReference, CellReference),
}
