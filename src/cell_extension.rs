use crate::expression_parser::Expr;
use std::collections::HashSet;

/// Represents an  cell in the spreadsheet.
///
/// This struct contains additional properties for a cell, such as its value, formula,
/// formatting options (bold, italics, underline), and dependency relationships.
#[derive(Clone)]
pub struct CellExtension {
    /// The value of the cell.
    pub value: i32,
    /// The formula associated with the cell, parsed as an expression.
    pub formula: Expr,
    /// The row index of the cell (0-based).
    pub r: i32,
    /// The column index of the cell (0-based).
    pub c: i32,
    /// Indicates whether the cell contains an error.
    pub is_error: bool,
    /// Indicates whether the cell needs to be recalculated.
    pub is_recalculate: bool,
    /// A set of cells that depend on this cell.
    pub dependents: HashSet<crate::cellsp::CellReference>,
    /// A set of cells that this cell depends on.
    pub precedents: HashSet<crate::cellsp::CellReference>,
    /// Indicates whether the cell's content is bold.
    pub is_bold: bool,
    /// Indicates whether the cell's content is italicized.
    pub is_italics: bool,
}

/// Represents the spreadsheet as a whole.
///
/// This struct contains the dimensions of the spreadsheet and all the cells within it.
#[derive(Clone)]
pub struct SpreadsheetExtension {
    /// The number of rows in the spreadsheet.
    pub rows: i32,
    /// The number of columns in the spreadsheet.
    pub columns: i32,
    /// A 2D vector containing all the cells in the spreadsheet.
    pub all_cells: Vec<Vec<CellExtension>>,
}

/// A type alias for `SpreadsheetExtension`.
///
/// This alias is used to simplify references to the spreadsheet structure.
pub type Sheet = SpreadsheetExtension;
