use crate::display::get_column_name;
use ::std::fmt;
use std::collections::HashMap;

/// Represents an operand in a formula.
///
/// An operand can either be a constant value or a reference to another cell.
#[derive(Clone)]
pub enum Operand {
    /// A constant integer value.
    Constant(i32),
    /// A reference to another cell in the spreadsheet.
    CellOperand(CellReference),
}

/// Represents a reference to a specific cell in the spreadsheet.
///
/// This struct is used to identify a cell by its row and column indices.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct CellReference {
    /// The row index of the cell (0-based).
    pub row: i32,
    /// The column index of the cell (0-based).
    pub column: i32,
}

impl fmt::Display for CellReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col_name = get_column_name(self.column); // Convert column index to name (e.g., 0 -> A)
        write!(f, "{}{}", col_name, self.row + 1) // Convert row index to 1-based and format as A1, B2, etc.
    }
}
/// Represents a cell in the spreadsheet.
///
/// This struct contains the value of the cell, its formula, and its dependency relationships.
pub struct Cell {
    /// The value of the cell.
    pub value: i32,
    /// The operation ID associated with the cell.
    pub operation_id: i32,
    /// The formula associated with the cell, represented as a vector of operands.
    pub formula: Vec<Operand>,
    /// The row index of the cell (0-based).
    pub r: i32,
    /// The column index of the cell (0-based).
    pub c: i32,
    /// Indicates whether the cell needs to be recalculated.
    pub is_recalculate: bool,
    /// Indicates whether the cell contains an error.
    pub is_error: bool,
    /// A map of cells that depend on this cell.
    ///
    /// The key is a `CellReference` to the dependent cell, and the value is a boolean indicating
    /// whether the dependency is active.
    pub dependents: HashMap<CellReference, bool>,
    /// A map of cells that this cell depends on.
    ///
    /// The key is a `CellReference` to the precedent cell, and the value is a boolean indicating
    /// whether the dependency is active.
    pub precedents: HashMap<CellReference, bool>,
}

/// Represents the spreadsheet.
///
/// This struct contains the dimensions of the spreadsheet and all the cells within it.
pub struct Spreadsheet {
    /// The number of rows in the spreadsheet.
    pub rows: i32,
    /// The number of columns in the spreadsheet.
    pub columns: i32,
    /// A 2D vector containing all the cells in the spreadsheet.
    pub all_cells: Vec<Vec<Cell>>,
}

/// A type alias for `Spreadsheet`.
///
/// This alias is used to simplify references to the spreadsheet structure.
pub type Sheet = Spreadsheet;
