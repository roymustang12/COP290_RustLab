use std::collections::HashSet;
use crate::expression_parser::Expr;


//THIS IS FOR EXTENSION

pub struct  CellExtension {
    pub value: i32,
    pub formula: Expr,
    pub r: i32,
    pub c: i32, 
    pub is_error: bool,
    pub is_recalculate: bool,
    pub dependents: HashSet<crate::cellsp::CellReference>,
    pub precedents: HashSet<crate::cellsp::CellReference>,
}

pub struct SpreadsheetExtension {
    pub rows: i32,
    pub columns: i32,
    pub all_cells: Vec<Vec<CellExtension>>,
}

pub type Sheet = SpreadsheetExtension;

