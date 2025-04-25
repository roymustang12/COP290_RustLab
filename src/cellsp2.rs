use std::collections::HashSet;

/// Represents an operand in a formula, which can either be a constant value or a reference to a cell.
#[derive(Clone, Debug)]
pub enum Operand {
    /// A constant integer value.
    Constant(i32),
    /// A reference to a cell in the spreadsheet.
    CellOperand(CellReference),
}

/// Represents a reference to a specific cell in the spreadsheet.
///
/// # Fields
///
/// * `row` - The row index of the cell.
/// * `column` - The column index of the cell.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct CellReference {
    pub row: i32,
    pub column: i32,
}

/// Represents a single cell in the spreadsheet.
///
/// # Fields
///
/// * `value` - The current value of the cell.
/// * `operation_id` - The ID of the operation assigned to the cell.
/// * `formula` - A vector of operands representing the formula in the cell.
/// * `r` - The row index of the cell.
/// * `c` - The column index of the cell.
/// * `is_recalculate` - A flag indicating whether the cell needs to be recalculated.
/// * `is_error` - A flag indicating whether the cell contains an error.
/// * `dependents` - A set of cells that depend on this cell.
/// * `precedents` - A set of cells that this cell depends on.
pub struct Cell {
    pub value: i32,
    pub operation_id: i32,
    pub formula: Vec<Operand>,
    pub r: i32,
    pub c: i32,
    pub is_recalculate: bool,
    pub is_error: bool,
    pub dependents: HashSet<CellReference>,
    pub precedents: HashSet<CellReference>,
}

/// Represents the entire spreadsheet.
///
/// # Fields
///
/// * `rows` - The number of rows in the spreadsheet.
/// * `columns` - The number of columns in the spreadsheet.
/// * `all_cells` - A 2D vector containing all the cells in the spreadsheet.
pub struct Spreadsheet {
    pub rows: i32,
    pub columns: i32,
    pub all_cells: Vec<Vec<Cell>>,
}

/// Alias for the `Spreadsheet` type.
pub type Sheet = Spreadsheet;
