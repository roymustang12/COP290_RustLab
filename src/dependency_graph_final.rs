

use std::{cell, collections::{btree_map::Values, HashMap, HashSet, VecDeque}};
use crate::cell::{Cell, CellReference, Operand, Sheet, Spreadsheet};
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use lazy_static::lazy_static;

lazy_static! {
    static ref SLEEP_CHANNEL: (Arc<Mutex<Sender<(i32, i32, i32)>>>, Arc<Mutex<Receiver<(i32, i32, i32)>>>) = {
        let (tx, rx) = mpsc::channel();
        (Arc::new(Mutex::new(tx)), Arc::new(Mutex::new(rx)))
    };
}

static mut STATUS: i32 = 0;

pub fn initialise(rows: i32, columns: i32) -> Spreadsheet {
    let mut all_cells = Vec::with_capacity(rows as usize);
    for r in 0..rows {
        let mut row: Vec<Cell> = Vec::with_capacity(columns as usize);
        for c in 0..columns {
            let mut curr_cell:Cell;
            curr_cell = Cell {
                value: 0,
                operation_id: -1,
                formula: Vec::new(),
                r: r,
                c: c,
                is_recalculate: false,
                is_error: false,
                dependents: HashMap::new(),
                precedents: HashMap::new(),
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

pub fn add_dependency(sheet: &mut Spreadsheet, rf: i32, cf: i32, rt: i32, ct: i32) {
    let dependent_cell = CellReference{row: rt, column: ct};
    let precedent_cell = CellReference{row:rf, column:cf};
    sheet.all_cells[rf as usize][cf as usize].dependents.insert(dependent_cell, true);
    sheet.all_cells[rt as usize][ct as usize].precedents.insert(precedent_cell, true);
}

pub fn delete_dependency(sheet: &mut Spreadsheet, rf: i32, cf: i32, rt: i32, ct: i32) {
    let dependent_cell = CellReference{row: rt, column: ct};
    //let precedent_cell = CellReference{row: rf, column: cf};
    sheet.all_cells[rf as usize][cf as usize]
        .dependents
        .remove(&dependent_cell);
}

pub fn clear_precedents(sheet: &mut Spreadsheet, rt: i32, ct: i32) {
    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
    cell.precedents = HashMap::new();
}

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
        },

        2 =>
        {   
            if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                let precedent_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_is_error {
                    cell.value = precedent_value;
                    cell.is_error = precedent_is_error;
                    unsafe {
                        STATUS = 0;
                    }
                }
                else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            
            }
        },

        3..=6 => {
            
            if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
                else {
                
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
                
            
            }
            let (value1, err1) = get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[0]);
            let (value2, err2) = get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[1]);
            if !err1 && !err2 {
                
                match sheet.all_cells[rt as usize][ct as usize].operation_id {
                    3 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 + value2;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        } // Addition
                    },
                    4 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 - value2; // Subtraction
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    },
                    5 => {
                        let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                        cell.value = value1 * value2; 
                        // Multiplication
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    },
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
                        return;
                    },
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
            
            if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;

                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
                else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            
            }
            let (values, is_error) =collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error == true {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            }
            else {
                sheet.all_cells[rt as usize][ct as usize].value = values.iter().min().cloned().unwrap_or(0);
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        },

        8 =>
        {
            
            if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                let precedent_cell_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                if !precedent_cell_is_error {
                    cell.value = precedent_cell_value;
                    cell.is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
                else {
                    cell.is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
            
            }
            let (values, is_error) = collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
            if is_error == true {
                sheet.all_cells[rt as usize][ct as usize].is_error = true;
                unsafe {
                    STATUS = 2;
                }
            }
            else {
                sheet.all_cells[rt as usize][ct as usize].value = values.iter().max().cloned().unwrap_or(0);
                sheet.all_cells[rt as usize][ct as usize].is_error = false;
                unsafe {
                    STATUS = 0;
                }
            }
        },

        9 => 
            {
                
                if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                    let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                    let precedent_cell_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                    if !precedent_cell_is_error {
                        cell.value = precedent_cell_value;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    else {
                        cell.is_error = true;
                        unsafe {
                            STATUS = 2;
                        }
                    }
                
                }
                let (values, is_error) = collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
                if is_error == true {
                    sheet.all_cells[rt as usize][ct as usize].is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
                else  {
                    let sum: i32 = values.iter().sum();
                    sheet.all_cells[rt as usize][ct as usize].value = sum / values.len() as i32;
                    sheet.all_cells[rt as usize][ct as usize].is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
            },
        
        10 =>
            {
                
                if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                    let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                    let  precedent_cell_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                    if !precedent_cell_is_error {
                        cell.value = precedent_cell_value;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    else {
                        cell.is_error = true;
                        unsafe {
                            STATUS = 2;
                        }
                    }
                
                }
                let (values, is_error) = collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
                if is_error == true {
                    sheet.all_cells[rt as usize][ct as usize].is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
                else  {
                    let sum: i32 = values.iter().sum();
                    sheet.all_cells[rt as usize][ct as usize].value = sum;
                    sheet.all_cells[rt as usize][ct as usize].is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
            },
        
        11 =>
            {
                
                if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                    let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                    let precedent_cell_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                    if !precedent_cell_is_error {
                        cell.value = precedent_cell_value;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    else {
                        cell.is_error = true;
                        unsafe {
                            STATUS = 2;
                        }
                    }
                
                }
                let (values, is_error) = collect_values(sheet, &sheet.all_cells[rt as usize][ct as usize].formula);
                if is_error == true {
                    sheet.all_cells[rt as usize][ct as usize].is_error = true;
                    unsafe {
                        STATUS = 2;
                    }
                }
                else  {
                    let stdev: i32 = stdev(&values);
                    sheet.all_cells[rt as usize][ct as usize].value = stdev;
                    sheet.all_cells[rt as usize][ct as usize].is_error = false;
                    unsafe {
                        STATUS = 0;
                    }
                }
            }
        
        12 =>
            {
                
                if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                    let precedent_cell_value = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                    let precedent_cell_is_error = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].is_error;
                    let cell = &mut sheet.all_cells[rt as usize][ct as usize];
                    if !precedent_cell_is_error {
                        cell.value = precedent_cell_value;
                        cell.is_error = false;
                        unsafe {
                            STATUS = 0;
                        }
                    }
                    else {
                        cell.is_error = true;
                        unsafe {
                            STATUS = 2;
                        }
                    }
                
                }
                if let Operand::Constant(value) = &sheet.all_cells[rt as usize][ct as usize].formula[0] {
                    handle_sleep(rt, ct, *value);
                }
                else {
                    if let Operand::CellOperand(cell_ref) = &sheet.all_cells[rt as usize][ct as usize].formula[0]
                        { 
                        let prcedent_cell = &sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize];
                        if prcedent_cell.is_error {
                            sheet.all_cells[rt as usize][ct as usize].is_error = true;
                            unsafe {
                                STATUS = 2;
                            }
                        }
                        else {
                            let seconds = sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value;
                            handle_sleep(rt, ct, seconds);
                        }
                    }
                }
            },

        _ => {
            panic!("Unknown operation");
        }

    }
}

pub fn get_operand_value(sheet: &Spreadsheet, operand: &Operand) -> (i32, bool) {
    match operand {
        Operand::Constant(value) => (*value, false),
        Operand::CellOperand(cell_ref) =>
                                        {
                                            let precedent_cell = &sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize];
                                            if precedent_cell.is_error {
                                                (precedent_cell.value, true)
                                            }
                                            else{
                                                (precedent_cell.value, false)
                                            }
                                        },
    }
} 

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

fn stdev(values: &Vec<i32>) -> i32 {
    if values.is_empty() {
        return 0;
    }
    
    let n = values.len() as f64;
    let mean: f64 = values.iter().map(|&x| x as f64).sum::<f64>() / n;
    
    let variance = values.iter()
        .map(|&x| {
            let diff = (x as f64) - mean;
            diff * diff
        })
        .sum::<f64>() / n;
    
    variance.sqrt() as i32
}

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

pub fn zero_div_err(sheet: &Spreadsheet, rt: i32, ct: i32) -> bool {
    let cell = &sheet.all_cells[rt as usize][ct as usize];
    if cell.operation_id == 6 && cell.precedents.len() == 2 {
        let (value, err) = get_operand_value(sheet, &sheet.all_cells[rt as usize][ct as usize].formula[1]);
        if value == 0 {
            return true;
        }
        else {
            return  false;
        }
    }
    else {
        false
    }
}

pub fn precedent_has_error(sheet: &Spreadsheet, rt: i32, ct: i32) -> bool {
    let precedents = &sheet.all_cells[rt as usize][ct as usize].precedents;
    let mut i = 0;
    for cell in precedents {
        let (value, err) = get_operand_value(sheet, &sheet.all_cells[cell.0.row as usize][cell.0.column as usize].formula[i]);
        i = i + 1;
        if err {
            return true;
        }
        else {

        }
    }
    false
}

pub fn dfs_cycle_detection(sheet: &Spreadsheet, rs: i32, cs: i32, visited: &mut HashSet<(i32, i32)>, recursion_stack: &mut HashSet<(i32, i32)>) -> bool {
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

    for (dependent, _) in &curr_cell.dependents {
        if dfs_cycle_detection(sheet,dependent.row, dependent.column, visited, recursion_stack) {
            return  true;
        }
    }

    recursion_stack.remove(&cell_coord);

    false

}

pub fn has_cycle(sheet: &Spreadsheet, row: i32, col: i32) -> bool {
    let mut visited:HashSet<(i32,i32)> = HashSet::new();
    let mut recursion_stack: HashSet<(i32, i32)> = HashSet::new();
    dfs_cycle_detection(sheet, row, col, &mut visited, &mut recursion_stack)
}

pub fn recalculate_dependents(sheet: &mut Spreadsheet, rs: i32, cs: i32) {
    let mut q: VecDeque<(i32, i32)> = VecDeque::new();
    {
        let cell = &sheet.all_cells[rs as usize][cs as usize];
        for (dependents, _) in &cell.dependents {
            q.push_back((dependents.row, dependents.column));
        }
    }

    while let Some((curr_r, curr_c)) = q.pop_front() {
        if zero_div_err(sheet, curr_r, curr_c) {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error = true;
            unsafe {
                STATUS = 2;
            }
        }
        else if precedent_has_error(sheet, curr_r, curr_c) {
            sheet.all_cells[curr_r as usize][curr_c as usize].is_error = true;
            unsafe {
                STATUS = 2;
            }
        }
        else {
            calculate_cell_value(sheet, curr_r, curr_c);
        }

        let curr_cell = &sheet.all_cells[curr_r as usize][curr_c as usize];
        for (dep_ref, _) in &curr_cell.dependents {
            q.push_back((dep_ref.row, dep_ref.column));
        }
    }
}

