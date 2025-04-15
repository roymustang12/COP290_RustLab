use crate::cell_extension::*;
use crate::cellsp::CellReference;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::expression_parser::Expr;
use crate::expression_utils::{eval_expr, extract_precedents};

use lazy_static::lazy_static;

lazy_static! {
    static ref SLEEP_CHANNEL: (
        Arc<Mutex<Sender<(i32, i32, i32)>>>,
        Arc<Mutex<Receiver<(i32, i32, i32)>>>
    ) = {
        let (tx, rx) = mpsc::channel();
        (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
    };
}

pub static mut STATUS_extension: i32 = 0;

pub fn initialise_extension(rows: i32, columns: i32) -> SpreadsheetExtension {
    let mut all_cells = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut row: Vec<CellExtension> = Vec::with_capacity(columns as usize);
        for c in 0..columns {
            let curr_cell = CellExtension {
                value: 0,
                // expression: None,
                // formula_str: String::new(),
                formula: Expr::Number(0),
                r: r,
                c: c,
                is_recalculate: false,
                is_error: false,
                dependents: HashSet::new(),
                precedents: HashSet::new(),
            };
            row.push(curr_cell);
        }
        all_cells.push(row);
    }

    SpreadsheetExtension {
        rows,
        columns,
        all_cells,
    }
}

pub fn add_dependency_extension(sheet: &mut SpreadsheetExtension, rf: i32, cf: i32, rt: i32, ct: i32) {
    let dependent_cell = CellReference {
        row: rt,
        column: ct,
    };
    let precedent_cell = CellReference {
        row: rf,
        column: cf,
    };
    sheet.all_cells[rf as usize][cf as usize]
        .dependents
        .insert(dependent_cell);
    sheet.all_cells[rt as usize][ct as usize]
        .precedents
        .insert(precedent_cell);
}

pub fn delete_dependency_extension(sheet: &mut SpreadsheetExtension, rf: i32, cf: i32, rt: i32, ct: i32) {
    let dependent_cell = CellReference {
        row: rt,
        column: ct,
    };
    sheet.all_cells[rf as usize][cf as usize]
        .dependents
        .remove(&dependent_cell);
}

pub fn clear_precedents_extension(sheet: &mut SpreadsheetExtension, rt: i32, ct: i32) {
    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
    cell.precedents = HashSet::new();
}

pub fn calculate_cell_value_extension(sheet: &mut SpreadsheetExtension, rt: i32, ct: i32) {
   
    let formula =  &sheet.all_cells[rt as usize][ct as usize].formula;
    let value  = eval_expr(formula, sheet);
    
    match formula {
        Expr::Number(_) => {
            sheet.all_cells[rt as usize][ct as usize].value = value;
            sheet.all_cells[rt as usize][ct as usize].is_error = false;
            unsafe {
                STATUS_extension = 0;
            }
        },

        Expr::Cell(cell_ref) => {
            if sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error {
                
                    sheet.all_cells[rt as usize][ct as usize].is_error = true;
                    unsafe {
                    STATUS_extension = 2;
                    }
                
            }
            else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_extension  = 0;
                }
            }
        },

        Expr::BinaryOp(left,op ,right ) => {
            if expr_has_error(formula, sheet) {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                STATUS_extension = 2;
                }
            }
            else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_extension = 0;
                }
            }
        },

        Expr::Function(_, args) => {
            if expr_has_error(formula, sheet) {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                STATUS_extension = 2;
                }
            }
            else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_extension = 0;
                }
            }
        },

        Expr::Range(_, _) => {
            unsafe {
                STATUS_extension = 12;
                //review this later
            }
        },


    }
    

}

pub fn expr_has_error(expr: &Expr, sheet: &SpreadsheetExtension) -> bool {
    match expr {
        Expr::Number(_) => {
            // Constants can't have errors
            false
        },
        
        Expr::Cell(cell_ref) => {
            // Check if the referenced cell has an error
            if cell_ref.row < 0 || cell_ref.row >= sheet.rows || 
               cell_ref.column < 0 || cell_ref.column >= sheet.columns {
                // Out of bounds reference is an error
                true
            } else {
                sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error
            }
        },
        
        Expr::BinaryOp(left, op, right) => {
            // Check if either operand has an error
            expr_has_error(left, sheet) || expr_has_error(right, sheet) || 
            // Special check for division by zero
            (*op == '/' && matches!(*(*right), Expr::Number(n) if n == 0))
        },
        
        Expr::Function(name, args) => {
            // Check if any function argument has an error
            for arg in args {
                if expr_has_error(arg, sheet) {
                    return true;
                }
            }
            false
        },
        
        Expr::Range(start, end) => {
            // Check all cells in the range for errors
            for r in start.row..=end.row {
                for c in start.column..=end.column {
                    if r < 0 || r >= sheet.rows || c < 0 || c >= sheet.columns {
                        return true; // Out of bounds
                    }
                    if sheet.all_cells[r as usize][c as usize].is_error {
                        return true;
                    }
                }
            }
            false
        }
    }
}

//Fuck you Sleep

pub fn zero_div_err_extension(sheet: &SpreadsheetExtension, rt: i32, ct: i32) -> bool {
    let formula = &sheet.all_cells[rt as usize][ct as usize].formula;
    check_division_by_zero(formula, sheet)
}

fn check_division_by_zero(expr: &Expr, sheet: &SpreadsheetExtension) -> bool {
    match expr {
        Expr::Number(_) => false,
        
        Expr::Cell(cell_ref) => sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error,
        
        Expr::BinaryOp(left, op, right) => {
            if *op == '/' {
                // Check if right operand evaluates to zero
                let right_value = eval_expr(right, sheet);
                if right_value == 0 {
                    return true;
                }
            }
            
            // Check both sides recursively
            check_division_by_zero(left, sheet) || check_division_by_zero(right, sheet)
        },
        
        Expr::Function(_, args) => {
            // Check all function arguments recursively
            for arg in args {
                if check_division_by_zero(arg, sheet) {
                    return true;
                }
            }
            false
        },
        
        Expr::Range(_, _) => false, // Range itself doesn't have division
    }
}


pub fn precedent_has_error_extension(sheet: &SpreadsheetExtension, rt: i32, ct: i32) -> bool {
    let precedents = &sheet.all_cells[rt as usize][ct as usize].precedents;
    for cell_ref in precedents {
        if sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error {
            return  true;
        }
        else {
            continue;
        }
    }
    false
}

pub fn dfs_cycle_detection(
    sheet: &SpreadsheetExtension,
    rs: i32,
    cs: i32,
    visited: &mut HashSet<(i32, i32)>,
    recursion_stack: &mut HashSet<(i32, i32)>,
) -> bool {
    let cell_coord = (rs, cs);
    if recursion_stack.contains(&cell_coord) {
        return true;
    }

    if visited.contains(&cell_coord) {
        return false;
    }

    visited.insert(cell_coord);
    recursion_stack.insert(cell_coord);

    let curr_cell = &sheet.all_cells[rs as usize][cs as usize];

    for (dependent) in &curr_cell.dependents {
        if dfs_cycle_detection(
            sheet,
            dependent.row,
            dependent.column,
            visited,
            recursion_stack,
        ) {
            return true;
        }
    }

    recursion_stack.remove(&cell_coord);                                        
    

    false
}

pub fn has_cycle(sheet: &SpreadsheetExtension, row: i32, col: i32) -> bool {
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut recursion_stack: HashSet<(i32, i32)> = HashSet::new();
    dfs_cycle_detection(sheet, row, col, &mut visited, &mut recursion_stack)
}

pub fn recalculate_dependents_extension(sheet: &mut SpreadsheetExtension, rs: i32, cs: i32) {
    let mut q:VecDeque<(i32, i32)> = VecDeque::new();
    {
        let cell = &sheet.all_cells[rs as usize][cs as usize];
        for dependent in &cell.dependents {
            q.push_back((dependent.row, dependent.column));
        }
    }

    while let Some((curr_r, curr_c)) = q.pop_front() {
        if zero_div_err_extension(sheet, curr_r, curr_c) {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error = true;
            unsafe {
                STATUS_extension = 2;
            }

        }
        else if precedent_has_error_extension(sheet, curr_r, curr_c) {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error  = true;
            unsafe {
                STATUS_extension = 2;
            }
        }
        else {
            calculate_cell_value_extension(sheet, curr_r, curr_c);
        }
        let curr_cell = &sheet.all_cells[curr_r as usize][curr_c as usize];
        for dep_ref in &curr_cell.dependents {
            q.push_back((dep_ref.row, dep_ref.column));
        }
    }
}

pub fn assign_cell_extension(
    sheet: &mut SpreadsheetExtension, 
    rt: i32,
    ct: i32,
    formula: Expr
) {
    let old_precedents = {
        let cell = &sheet.all_cells[rt as usize][ct as usize];
        cell.precedents.clone()
    };

    for prec_ref in &old_precedents {
        delete_dependency_extension(sheet, prec_ref.row, prec_ref.column, rt, ct);
    }

    clear_precedents_extension(sheet, rt, ct);
    
    let old_formula = {
        let cell = &sheet.all_cells[rt as usize][ct as usize];
        cell.formula.clone()
    };

    {
        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
        cell.formula = formula.clone();
    }

    let precedents = extract_precedents(&formula);
    for cell_ref in &precedents {
        add_dependency_extension(sheet, cell_ref.row, cell_ref.column, rt, ct);
    }

    if !has_cycle(sheet, rt, ct) {
        if !zero_div_err_extension(sheet, rt, ct) {
            calculate_cell_value_extension(sheet, rt, ct);
            recalculate_dependents_extension(sheet, rt, ct);
            unsafe {
                STATUS_extension = 0;
            }
        }
        else {
            sheet.all_cells[rt as usize][ct as usize].is_error = true;
            unsafe {
                STATUS_extension = 2;
            }
            recalculate_dependents_extension(sheet, rt, ct);
        }
    } else {
        unsafe {
            STATUS_extension = 3;
        }
        sheet.all_cells[rt as usize][ct as usize].formula = old_formula;
        
        for old_precedent in &old_precedents{
            add_dependency_extension(sheet, old_precedent.row, old_precedent.column, rt, ct);
        }

        let new_precedents = extract_precedents(&formula);
        for new_precedent in &new_precedents {
            delete_dependency_extension(sheet, new_precedent.row, new_precedent.column, rt, ct);
        }
        
    }

}