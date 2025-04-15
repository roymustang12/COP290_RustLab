use crate::cellsp::{CellReference};
use std::str;

//THIS IS FOR EXTENSION
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Cell(CellReference),
    BinaryOp(Box<Expr>, char, Box<Expr>),
    Function(String, Vec<Expr>),
    Range(CellReference, CellReference),
}

