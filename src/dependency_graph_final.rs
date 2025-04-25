use crate::cellsp2::{Cell, CellReference, Operand, Spreadsheet};
use lazy_static::lazy_static;
use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

type CellCoord = (i32, i32, i32);
type ThreadSafeSender = Arc<Mutex<Sender<CellCoord>>>;
type ThreadSafeReceiver = Arc<Mutex<Receiver<CellCoord>>>;

lazy_static! {
    static ref SLEEP_CHANNEL: (ThreadSafeSender, ThreadSafeReceiver) = {
        let (tx, rx) = channel::<CellCoord>();
        (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
    };
}

pub static mut STATUS: i32 = 0;

/// Initializes a spreadsheet with the given number of rows and columns.
///
/// # Arguments
///
/// * `rows` - The number of rows in the spreadsheet.
/// * `columns` - The number of columns in the spreadsheet.
///
/// # Returns
///
/// A `Spreadsheet` instance with all cells initialized.
pub fn initialise(rows: i32, columns: i32) -> Spreadsheet {
    let mut all_cells = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut row: Vec<Cell> = Vec::with_capacity(columns as usize);
        for c in 0..columns {
            let curr_cell = Cell {
                value: 0,
                operation_id: -1,
                formula: Vec::new(),
                r,
                c,
                is_recalculate: false,
                is_error: false,
                dependents: HashSet::new(),
                precedents: HashSet::new(),
            };

            row.push(curr_cell);
        }
        all_cells.push(row);
    }

    Spreadsheet {
        rows,
        columns,
        all_cells,
    }
}

/// Adds a dependency between two cells in the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rf` - Row index of the precedent cell.
/// * `cf` - Column index of the precedent cell.
/// * `rt` - Row index of the dependent cell.
/// * `ct` - Column index of the dependent cell.
pub fn add_dependency(sheet: &mut Spreadsheet, rf: i32, cf: i32, rt: i32, ct: i32) {
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
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rf` - Row index of the precedent cell.
/// * `cf` - Column index of the precedent cell.
/// * `rt` - Row index of the dependent cell.
/// * `ct` - Column index of the dependent cell.
pub fn delete_dependency(sheet: &mut Spreadsheet, rf: i32, cf: i32, rt: i32, ct: i32) {
    let dependent_cell = CellReference {
        row: rt,
        column: ct,
    };
    //let precedent_cell = CellReference{row: rf, column: cf};
    sheet.all_cells[rf as usize][cf as usize]
        .dependents
        .remove(&dependent_cell);
}

/// Clears all precedents of a specific cell in the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - Row index of the target cell.
/// * `ct` - Column index of the target cell.
pub fn clear_precedents(sheet: &mut Spreadsheet, rt: i32, ct: i32) {
    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
    cell.precedents = HashSet::new();
}

/// Calculates the value of a specific cell in the spreadsheet based on the formula specified.Before assigning the value,it checks if the update is to validated.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - Row index of the target cell.
/// * `ct` - Column index of the target cell.
pub fn calculate_cell_value(sheet: &mut Spreadsheet, rt: i32, ct: i32) {
    // let cell = &mut sheet.all_cells[rt as usize][ct as usize];
    match sheet.all_cells[rt as usize][ct as usize].operation_id {
        1 => {
            let cell = &mut sheet.all_cells[rt as usize][ct as usize];

            if let Operand::Constant(value) = &cell.formula[0] {
                cell.value = *value;
                cell.is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        2 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_is_error {
                    cell.value = precedent_value;
                    cell.is_error = precedent_is_error;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
        }

        3..=6 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (value1, err1) =
                get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[0]);
            let (value2, err2) =
                get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[1]);
            if !err1 && !err2 {
                match sheet.all_cells[rt as usize][ct as usize].operation_id {
                    3 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 + value2;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        } // Addition
                    }
                    4 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 - value2; // Subtraction
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    5 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 * value2;
                        // Multiplication
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    6 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        // Division (check for division by zero)
                        if value2 == 0 {
                            cell.is_error = true;
                            unsafe {
                                STATUS = 2;
                            }
                        } else {
                            cell.value = value1 / value2;
                            cell.is_error = false;
                            unsafe {
                                STATUS = 0;
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            }
        }

        7 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;

                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (values, is_error) =
                collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].value =
                    values.iter().min().cloned().unwrap_or(0);
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        8 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (values, is_error) =
                collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            } else {
                sheet.all_cells[rt as usize][ct as usize].value =
                    values.iter().max().cloned().unwrap_or(0);
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        9 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (values, is_error) =
                collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            } else {
                let sum: i32 = values.iter().sum();
                sheet.all_cells[rt as usize][ct as usize].value = sum / values.len() as i32;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        10 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (values, is_error) =
                collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            } else {
                let sum: i32 = values.iter().sum();
                sheet.all_cells[rt as usize][ct as usize].value = sum;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        11 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            let (values, is_error) =
                collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            } else {
                let stdev: i32 = stdev(&values);
                sheet.all_cells[rt as usize][ct as usize].value = stdev;
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        }

        12 => {
            if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let precedent_cell_value =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error =
                    sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                } else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            }
            if let Operand::Constant(value) = &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                handle_sleep(rt, ct, *value);
            } else if let Operand::CellOperand(cell_ref) =
                &sheet.all_cells[rt as usize][ct as usize].formula[0]
            {
                let prcedent_cell =
                    &sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize];
                if prcedent_cell.is_error {
                    sheet.all_cells[rt as usize][ct as usize].is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                } else {
                    let seconds =
                        sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                    handle_sleep(rt, ct, seconds);
                }
            }
        }
        _ => {
            panic!("Unknown operation");
        }
    }
}

/// Retrieves the value of an operand in the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `operand` - The operand to evaluate.
///
/// # Returns
///
/// A tuple containing the value of the operand and a boolean indicating if there was an error.
pub fn get_operand_value(sheet: &Spreadsheet, operand: &Operand) -> (i32, bool) {
    match operand {
        Operand::Constant(value) => (*value, false),
        Operand::CellOperand(cell_ref) => {
            let precedent_cell = &sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize];
            if precedent_cell.is_error {
                (precedent_cell.value, true)
            } else {
                (precedent_cell.value, false)
            }
        }
    }
}

/// Collects values from a formula in the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `formula` - A vector of operands representing the formula.
///
/// # Returns
///
/// A tuple containing a vector of values and a boolean indicating if there was an error.
pub fn collect_values(sheet: &Spreadsheet, formula: &Vec<Operand>) -> (Vec<i32>, bool) {
    let mut is_error = false;
    let mut values = Vec::new();
    for operand in formula {
        let (curr_value, error) = get_operand_value(sheet, operand);
        if error {
            is_error = true;
        }
        values.push(curr_value);
    }
    (values, is_error)
}

/// Calculates the standard deviation of a vector of integers.
///
/// # Arguments
///
/// * `values` - A reference to a vector of integers.
///
/// # Returns
///
/// The standard deviation of the values as an integer. If the vector has one or fewer elements, it returns `0` to avoid division by zero.
///
/// # Notes
///
/// The calculation uses integer division for the mean and rounds the final standard deviation to the nearest integer.
fn stdev(values: &Vec<i32>) -> i32 {
    let n = values.len();
    if n <= 1 {
        return 0; // Avoid division by zero
    }

    // Calculate mean using integer division
    let sum: i32 = values.iter().sum();
    let mean = sum / n as i32;

    // Calculate variance
    let mut variance = 0.0;
    for &value in values {
        variance += ((value - mean) * (value - mean)) as f64;
    }
    variance /= n as f64;

    // Return integer standard deviation (rounded)
    (variance.sqrt().round()) as i32
}

/// Handles a sleep operation for a specific cell.
///
/// # Arguments
///
/// * `row` - Row index of the target cell.
/// * `col` - Column index of the target cell.
/// * `seconds` - The duration of the sleep operation in seconds.
///
/// # Returns
///
/// The duration of the sleep operation.
pub fn handle_sleep(row: i32, col: i32, seconds: i32) -> i32 {
    let tx = SLEEP_CHANNEL.0.clone();
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(seconds as u64));
        if let Ok(tx) = tx.lock() {
            let _ = tx.send((row, col, seconds));
        }
    });
    seconds
}

/// Processes completed sleep operations and updates the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
pub fn process_sleep_completions(sheet: &mut Spreadsheet) {
    if let Ok(rx) = SLEEP_CHANNEL.1.lock() {
        while let Ok((row, col, seconds)) = rx.try_recv() {
            sheet.all_cells[row as usize][col as usize].value = seconds;
            sheet.all_cells[row as usize][col as usize].is_error = false;
            unsafe {
                STATUS = 0;
            }
        }
    }
}

/// Checks if a division by zero error exists for a specific cell.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `rt` - Row index of the target cell.
/// * `ct` - Column index of the target cell.
///
/// # Returns
///
/// `true` if a division by zero error exists, otherwise `false`.
pub fn zero_div_err(sheet: &Spreadsheet, rt: i32, ct: i32) -> bool {
    let cell = &sheet.all_cells[rt as usize][ct as usize];
    if cell.operation_id == 6 && cell.precedents.len() == 2 {
        let (value, _) =
            get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[1]);
        value == 0
    } else {
        false
    }
}

/// Checks if any precedent of a specific cell has an error.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `rt` - Row index of the target cell.
/// * `ct` - Column index of the target cell.
///
/// # Returns
///
/// `true` if any precedent has an error, otherwise `false`.
pub fn precedent_has_error(sheet: &Spreadsheet, rt: i32, ct: i32) -> bool {
    let precedents = &sheet.all_cells[rt as usize][ct as usize].precedents;
    let mut _i = 0;
    for cell in precedents {
        let err = sheet.all_cells[cell.row as usize][cell.column as usize].is_error;
        if err {
            return true;
        }
    }
    false
}

/// Performs a depth-first search to detect cycles in the dependency graph.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `rs` - Row index of the starting cell.
/// * `cs` - Column index of the starting cell.
/// * `visited` - A mutable reference to the set of visited cells.
/// * `recursion_stack` - A mutable reference to the recursion stack.
///
/// # Returns
///
/// `true` if a cycle is detected, otherwise `false`.
pub fn dfs_cycle_detection(
    sheet: &Spreadsheet,
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

/// Checks if a cycle exists in the dependency graph for a specific cell.
///
/// # Arguments
///
/// * `sheet` - A reference to the spreadsheet.
/// * `row` - Row index of the target cell.
/// * `col` - Column index of the target cell.
///
/// # Returns
///
/// `true` if a cycle exists, otherwise `false`.
pub fn has_cycle(sheet: &Spreadsheet, row: i32, col: i32) -> bool {
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut recursion_stack: HashSet<(i32, i32)> = HashSet::new();
    dfs_cycle_detection(sheet, row, col, &mut visited, &mut recursion_stack)
}

/// Recalculates the values of all dependents of a specific cell.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rs` - Row index of the starting cell.
/// * `cs` - Column index of the starting cell.
pub fn recalculate_dependents(sheet: &mut Spreadsheet, rs: i32, cs: i32) {
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();
    {
        let cell = &sheet.all_cells[rs as usize][cs as usize];
        for dependents in &cell.dependents {
            q.push_back((dependents.row, dependents.column));
        }
    }

    while let Some((curr_r, curr_c)) = q.pop_front() {
        if zero_div_err(sheet, curr_r, curr_c) || precedent_has_error(sheet, curr_r, curr_c) {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error = true;
            unsafe {
                STATUS = 2;
            }
        } else {
            calculate_cell_value(sheet, curr_r, curr_c);
        }

        let curr_cell = &sheet.all_cells[curr_r as usize][curr_c as usize];
        for dep_ref in &curr_cell.dependents {
            q.push_back((dep_ref.row, dep_ref.column));
        }
    }
}

/// Assigns a formula and operation to a specific cell in the spreadsheet.
///
/// # Arguments
///
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `rt` - Row index of the target cell.
/// * `ct` - Column index of the target cell.
/// * `operation_id` - The operation ID to assign.
/// * `formula` - A vector of operands representing the formula.
pub fn assign_cell(
    sheet: &mut Spreadsheet,
    rt: i32,
    ct: i32,
    operation_id: i32,
    formula: Vec<Operand>,
) {
    let temp_precedents: HashSet<CellReference>;

    {
        let target_cell = &mut sheet.all_cells[rt as usize][ct as usize];
        temp_precedents = target_cell.precedents.clone();
    }
    //dependents of target_cell do not change
    //precedents will be cleared
    for cell_ref in &temp_precedents {
        delete_dependency(sheet, cell_ref.row, cell_ref.column, rt, ct);
    }
    clear_precedents(sheet, rt, ct);
    let temp_formula = formula.clone();
    //WARNING do not change the dependents
    sheet.all_cells[rt as usize][ct as usize].formula = formula;
    sheet.all_cells[rt as usize][ct as usize].operation_id = operation_id;
    for precedent in &temp_formula {
        if let Operand::CellOperand(cell_ref) = precedent {
            add_dependency(sheet, cell_ref.row, cell_ref.column, rt, ct);
        }
    }

    if !has_cycle(sheet, rt, ct) {
        if !zero_div_err(sheet, rt, ct) {
            calculate_cell_value(sheet, rt, ct);
            recalculate_dependents(sheet, rt, ct);
            unsafe {
                STATUS = 0;
            }
        } else {
            sheet.all_cells[rt as usize][ct as usize].is_error = true;
            unsafe { STATUS = 2 }
            recalculate_dependents(sheet, rt, ct);
        }
    } else {
        unsafe {
            STATUS = 3;
        }
        sheet.all_cells[rt as usize][ct as usize].operation_id = operation_id;
        sheet.all_cells[rt as usize][ct as usize].formula = temp_formula.clone();

        for old_precedent in &temp_precedents {
            add_dependency(sheet, old_precedent.row, old_precedent.column, rt, ct);
        }
        for new_precedent in &temp_formula {
            if let Operand::CellOperand(cell_ref) = new_precedent {
                delete_dependency(sheet, cell_ref.row, cell_ref.column, rt, ct);
            }
        }
    }
}
