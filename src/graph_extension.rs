use crate::cell_extension::*;
use crate::cellsp::CellReference;
use crate::expression_parser::Expr;
use crate::expression_utils::{eval_expr, extract_precedents};
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

type ThreadSafeSender = Arc<Mutex<Sender<(i32, i32, i32)>>>;
type ThreadSafeReceiver = Arc<Mutex<Receiver<(i32, i32, i32)>>>;

lazy_static! {
    static ref SLEEP_CHANNEL: (ThreadSafeSender, ThreadSafeReceiver) = {
        let (tx, rx) = mpsc::channel();
        (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
    };
}

/// A global status variable for tracking errors in the spreadsheet.
pub static mut STATUS_EXTENSION: i32 = 0;

const MAX_UNDO: usize = 17;

/// Initializes the spreadsheet with the given number of rows and columns.
///
/// # Arguments
/// * `rows` - The number of rows in the spreadsheet.
/// * `columns` - The number of columns in the spreadsheet.
///
/// # Returns
/// A `SpreadsheetExtension` object with all cells initialized.
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
                r,
                c,
                is_recalculate: false,
                is_error: false,
                dependents: HashSet::new(),
                precedents: HashSet::new(),
                is_bold: false,
                is_italics: false,
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

///Adds a dependency between two cells in the spreadsheet.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rf` - The row index of the precedent cell.
/// * `cf` - The column index of the precedent cell.
/// * `rt` - The row index of the dependent cell.
/// * `ct` - The column index of the dependent cell.
pub fn add_dependency_extension(
    sheet: &mut SpreadsheetExtension,
    rf: i32,
    cf: i32,
    rt: i32,
    ct: i32,
) {
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

/// Deletes a dependency between two cells in the spreadsheet.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rf` - The row index of the precedent cell.
/// * `cf` - The column index of the precedent cell.
/// * `rt` - The row index of the dependent cell.
/// * `ct` - The column index of the dependent cell.
pub fn delete_dependency_extension(
    sheet: &mut SpreadsheetExtension,
    rf: i32,
    cf: i32,
    rt: i32,
    ct: i32,
) {
    let dependent_cell = CellReference {
        row: rt,
        column: ct,
    };
    sheet.all_cells[rf as usize][cf as usize]
        .dependents
        .remove(&dependent_cell);
}

/// Clears all precedents of a given cell.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - The row index of the cell.
/// * `ct` - The column index of the cell.
pub fn clear_precedents_extension(sheet: &mut SpreadsheetExtension, rt: i32, ct: i32) {
    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
    cell.precedents = HashSet::new();
}

/// Calculates the value of a cell based on its formula.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - The row index of the cell.
/// * `ct` - The column index of the cell.
///
/// # Behavior
/// Updates the cell's value and error status based on its formula.
pub fn calculate_cell_value_extension(sheet: &mut SpreadsheetExtension, rt: i32, ct: i32) {
    let formula = &sheet.all_cells[rt as usize][ct as usize].formula;
    let value = eval_expr(formula, sheet);

    match formula {
        Expr::Number(_) => {
            sheet.all_cells[rt as usize][ct as usize].value = value;
            sheet.all_cells[rt as usize][ct as usize].is_error = false;
            unsafe {
                STATUS_EXTENSION = 0;
            }
        }

        Expr::Cell(cell_ref) => {
            if sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS_EXTENSION = 2;
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_EXTENSION = 0;
                }
            }
        }

        Expr::BinaryOp(_, _, _) => {
            if expr_has_error(formula, sheet) {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS_EXTENSION = 2;
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_EXTENSION = 0;
                }
            }
        }

        Expr::Function(_, _) => {
            if expr_has_error(formula, sheet) {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS_EXTENSION = 2;
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].value = value;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS_EXTENSION = 0;
                }
            }
        }

        Expr::Range(_, _) => {
            unsafe {
                STATUS_EXTENSION = 12;
                //review this later
            }
        }
    }
}

/// Checks if an expression has an error.
///
/// # Arguments
/// * `expr` - A reference to the expression.
/// * `sheet` - A reference to the spreadsheet.
///
/// # Returns
/// `true` if the expression has an error, `false` otherwise.
pub fn expr_has_error(expr: &Expr, sheet: &SpreadsheetExtension) -> bool {
    match expr {
        Expr::Number(_) => {
            // Constants can't have errors
            false
        }

        Expr::Cell(cell_ref) => {
            // Check if the referenced cell has an error
            if cell_ref.row < 0
                || cell_ref.row >= sheet.rows
                || cell_ref.column < 0
                || cell_ref.column >= sheet.columns
            {
                // Out of bounds reference is an error
                true
            } else {
                sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error
            }
        }

        Expr::BinaryOp(left, op, right) => {
            // Check if either operand has an error
            expr_has_error(left, sheet)
                || expr_has_error(right, sheet)
                || (*op == '/' && matches!(*(*right), Expr::Number(n) if n == 0))
        }

        Expr::Function(_, args) => {
            // Check if any function argument has an error
            for arg in args {
                if expr_has_error(arg, sheet) {
                    return true;
                }
            }
            false
        }

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

/// Checks if a division by zero occurs in the formula of a specific cell.
///
/// # Arguments
/// * `sheet` - A reference to the spreadsheet.
/// * `rt` - The row index of the cell.
/// * `ct` - The column index of the cell.
///
/// # Returns
/// `true` if a division by zero is detected, `false` otherwise.
pub fn zero_div_err_extension(sheet: &SpreadsheetExtension, rt: i32, ct: i32) -> bool {
    let formula = &sheet.all_cells[rt as usize][ct as usize].formula;
    check_division_by_zero(formula, sheet)
}

/// Recursively checks if a division by zero occurs in an expression.
///
/// # Arguments
/// * `expr` - A reference to the expression to analyze.
/// * `sheet` - A reference to the spreadsheet.
///
/// # Returns
/// `true` if a division by zero is detected, `false` otherwise.
fn check_division_by_zero(expr: &Expr, sheet: &SpreadsheetExtension) -> bool {
    match expr {
        Expr::Number(_) => false,

        Expr::Cell(cell_ref) => {
            sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error
        }

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
        }

        Expr::Function(_, args) => {
            // Check all function arguments recursively
            for arg in args {
                if check_division_by_zero(arg, sheet) {
                    return true;
                }
            }
            false
        }

        Expr::Range(_, _) => false, // Range itself doesn't have division
    }
}

/// Checks if any of the precedents of a specific cell have an error.
///
/// # Arguments
/// * `sheet` - A reference to the spreadsheet.
/// * `rt` - The row index of the cell.
/// * `ct` - The column index of the cell.
///
/// # Returns
/// `true` if any precedent of the cell has an error, `false` otherwise.
pub fn precedent_has_error_extension(sheet: &SpreadsheetExtension, rt: i32, ct: i32) -> bool {
    let precedents = &sheet.all_cells[rt as usize][ct as usize].precedents;
    for cell_ref in precedents {
        if sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error {
            return true;
        } else {
            continue;
        }
    }
    false
}
/// Performs a depth-first search (DFS) to detect cycles in the dependency graph.
///
/// # Arguments
/// * `sheet` - A reference to the spreadsheet.
/// * `rs` - The row index of the current cell.
/// * `cs` - The column index of the current cell.
/// * `visited` - A mutable set of visited cells.
/// * `recursion_stack` - A mutable set representing the current recursion stack.
///
/// # Returns
/// `true` if a cycle is detected, `false` otherwise.
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

    for dependent in &curr_cell.dependents {
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

/// Checks if a cycle exists in the dependency graph starting from a specific cell.
///
/// # Arguments
/// * `sheet` - A reference to the spreadsheet.
/// * `row` - The row index of the starting cell.
/// * `col` - The column index of the starting cell.
///
/// # Returns
/// `true` if a cycle is detected, `false` otherwise.
pub fn has_cycle(sheet: &SpreadsheetExtension, row: i32, col: i32) -> bool {
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut recursion_stack: HashSet<(i32, i32)> = HashSet::new();
    dfs_cycle_detection(sheet, row, col, &mut visited, &mut recursion_stack)
}

/// Recalculates the values of all dependent cells in the spreadsheet.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rs` - The row index of the starting cell.
/// * `cs` - The column index of the starting cell.
///
/// # Behavior
/// Updates the values of all dependent cells and propagates errors if necessary.
pub fn recalculate_dependents_extension(sheet: &mut SpreadsheetExtension, rs: i32, cs: i32) {
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();
    {
        let cell = &sheet.all_cells[rs as usize][cs as usize];
        for dependent in &cell.dependents {
            q.push_back((dependent.row, dependent.column));
        }
    }

    while let Some((curr_r, curr_c)) = q.pop_front() {
        if zero_div_err_extension(sheet, curr_r, curr_c)
            || precedent_has_error_extension(sheet, curr_r, curr_c)
        {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error = true;
            unsafe {
                STATUS_EXTENSION = 2;
            }
        } else {
            calculate_cell_value_extension(sheet, curr_r, curr_c);
        }
        let curr_cell = &sheet.all_cells[curr_r as usize][curr_c as usize];
        for dep_ref in &curr_cell.dependents {
            q.push_back((dep_ref.row, dep_ref.column));
        }
    }
}

/// Assigns a formula to a cell and updates its value and dependencies.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - The row index of the cell.
/// * `ct` - The column index of the cell.
/// * `formula` - The formula to assign to the cell.
///
/// # Behavior
/// * Updates the cell's formula and recalculates its value.
/// * Updates the dependency graph and propagates changes to dependent cells.
/// * Detects and handles cycles in the dependency graph.
pub fn assign_cell_extension(
    sheet: &mut SpreadsheetExtension,
    undo_manager: &mut UndoRedoStack,
    rt: i32,
    ct: i32,
    formula: Expr,
) {
    let old_state = CellState {
        row: rt,
        column: ct,
        is_error: sheet.all_cells[rt as usize][ct as usize].is_error,
        formula: sheet.all_cells[rt as usize][ct as usize].formula.clone(),
        precedents: sheet.all_cells[rt as usize][ct as usize].precedents.clone(),
    };

    undo_manager.push_state(old_state);

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
                STATUS_EXTENSION = 0;
            }
        } else {
            sheet.all_cells[rt as usize][ct as usize].is_error = true;
            unsafe {
                STATUS_EXTENSION = 2;
            }
            recalculate_dependents_extension(sheet, rt, ct);
        }
    } else {
        unsafe {
            STATUS_EXTENSION = 3;
        }
        sheet.all_cells[rt as usize][ct as usize].formula = old_formula;

        for old_precedent in &old_precedents {
            add_dependency_extension(sheet, old_precedent.row, old_precedent.column, rt, ct);
        }

        let new_precedents = extract_precedents(&formula);
        for new_precedent in &new_precedents {
            delete_dependency_extension(sheet, new_precedent.row, new_precedent.column, rt, ct);
        }
    }
}
#[derive(Clone)]
pub struct CellState {
    row: i32,
    column: i32,
    formula: Expr,
    is_error: bool,
    precedents: HashSet<CellReference>,
}

//When an assignment is done to a cell push onto the undo stack
//When an undo is done pop off the undo stack and push onto the redo stack
#[derive(Clone)]
pub struct UndoRedoStack {
    undo_stack: Vec<CellState>,
    redo_stack: Vec<CellState>,
}

impl Default for UndoRedoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoRedoStack {
    pub fn new() -> Self {
        UndoRedoStack {
            undo_stack: Vec::with_capacity(MAX_UNDO),
            redo_stack: Vec::new(),
        }
    }

    pub fn push_state(&mut self, state: CellState) {
        self.undo_stack.push(state);

        // self.redo_stack.clear();

        // self.redo_stack.clear();

        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn pop_undo(&mut self) -> Option<CellState> {
        self.undo_stack.pop()
    }

    pub fn push_redo(&mut self, state: CellState) {
        self.redo_stack.push(state);
    }

    pub fn pop_redo(&mut self) -> Option<CellState> {
        self.redo_stack.pop()
    }
}

pub fn perform_undo(sheet: &mut SpreadsheetExtension, undo_manager: &mut UndoRedoStack) -> bool {
    if let Some(old_state) = undo_manager.pop_undo() {
        let current_state = CellState {
            row: old_state.row,
            column: old_state.column,
            formula: sheet.all_cells[old_state.row as usize][old_state.column as usize]
                .formula
                .clone(),
            precedents: sheet.all_cells[old_state.row as usize][old_state.column as usize]
                .precedents
                .clone(),
            is_error: sheet.all_cells[old_state.row as usize][old_state.column as usize].is_error,
        };

        undo_manager.push_redo(current_state);

        restore_cell_state(sheet, old_state);
        true
    } else {
        false
    }
}

pub fn perform_redo(sheet: &mut SpreadsheetExtension, undo_manager: &mut UndoRedoStack) -> bool {
    if let Some(redo_state) = undo_manager.pop_redo() {
        let current_state = CellState {
            row: redo_state.row,
            column: redo_state.column,
            is_error: redo_state.is_error,
            formula: sheet.all_cells[redo_state.row as usize][redo_state.column as usize]
                .formula
                .clone(),
            precedents: sheet.all_cells[redo_state.row as usize][redo_state.column as usize]
                .precedents
                .clone(),
        };

        restore_cell_state(sheet, redo_state);

        undo_manager.push_state(current_state);

        true
    } else {
        false
    }
}

pub fn restore_cell_state(sheet: &mut SpreadsheetExtension, state: CellState) {
    let precedents = sheet.all_cells[state.row as usize][state.column as usize]
        .precedents
        .clone();

    for prec_ref in precedents {
        delete_dependency_extension(
            sheet,
            prec_ref.row,
            prec_ref.column,
            state.row,
            state.column,
        );
    }

    clear_precedents_extension(sheet, state.row, state.column);

    sheet.all_cells[state.row as usize][state.column as usize].formula = state.formula;

    for prec_ref in &state.precedents {
        add_dependency_extension(
            sheet,
            prec_ref.row,
            prec_ref.column,
            state.row,
            state.column,
        );
    }

    calculate_cell_value_extension(sheet, state.row, state.column);

    recalculate_dependents_extension(sheet, state.row, state.column);
}
