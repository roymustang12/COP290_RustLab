use std::collections::HashMap;

pub enum Operand {
    Constant(i32),
    CellOperand(CellReference),
}

#[derive(Eq, Hash, PartialEq)]
pub struct CellReference {
    pub row: i32,
    pub column: i32,
}

pub struct Cell {
    pub value: i32,
    pub operation_id: i32,
    pub formula: Vec<Operand>,
    pub r: i32,
    pub c: i32,
    pub is_recalculate: bool,
    pub is_error: bool,
    pub dependents: HashMap<CellReference, bool>,
    pub precedents: HashMap<CellReference, bool>,
}

pub struct Spreadsheet {
    pub rows: i32,
    pub columns: i32,
    pub all_cells: Vec<Vec<Cell>>,
}

pub type Sheet = Spreadsheet;
