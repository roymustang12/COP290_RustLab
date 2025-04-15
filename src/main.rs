use std::cmp::{max, min};
use std::collections::HashMap;
use std::ffi::CString;
use std::io::{self, Write};
use std::process;
use std::ptr;
use std::time::Instant;

use rust_lab::cellsp::Operand;
use rust_lab::cellsp::Spreadsheet;
use rust_lab::dependency_graph_final::assign_cell;
use rust_lab::dependency_graph_final::initialise;
use rust_lab::dependency_graph_final::STATUS;
use rust_lab::input::parse_cell_name;
use rust_lab::input::parse_input;

const MAX_ROWS: i32 = 999;
const MAX_COLS: i32 = 18278;
const VIEWPORT_SIZE: i32 = 10;

static mut CURRENT_ROW: i32 = 0;
static mut CURRENT_COL: i32 = 0;
static mut DISPLAY_CELL: i32 = 0;
static mut DISPLAY_ROW: i32 = 0;
static mut DISPLAY_COLUMN: i32 = 0;
// static mut STATUS: i32 = 0;
// static mut EDIT_ROW: usize = 0;
// static mut EDIT_COLUMN: usize = 0;
// static mut OPERATION_ID: i32 = 0;
// static mut FORMULA: Vec<Operand> = Vec::new();
// static mut COUNT_OPERANDS: usize = 0;

// #[derive(Clone)]
// pub struct Operand {
//     // Define the Operand structure based on your C code
// }

// pub struct Cell {
//     value: i32,
//     is_error: bool,
//     // Add other fields as per your C struct
// }

// pub struct Sheet {
//     rows: usize,
//     columns: usize,
//     all_cells: Vec<Vec<Cell>>,
// }

// impl Sheet {
//     fn new(rows: usize, columns: usize) -> Self {
//         let cell = Cell {
//             value: 0,
//             is_error: false,
//             // Initialize other fields as necessary
//         };
//         let all_cells = vec![vec![cell; columns]; rows];
//         Sheet { rows, columns, all_cells }
//     }
// }

fn get_col_label(col: i32) -> String {
    let mut col = col + 1; // 1-based index
    let mut label = String::new();
    while col > 0 {
        col -= 1;
        label.push((b'A' + (col % 26) as u8) as char);
        col /= 26;
    }
    label.chars().rev().collect()
}

fn display_sheet(sheet: &Spreadsheet) {
    unsafe {
        print!("   ");
        for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns as i32) {
            print!("{:^9}", get_col_label(c));
        }
        println!();

        for r in CURRENT_ROW..min(CURRENT_ROW + VIEWPORT_SIZE, sheet.rows as i32) {
            print!("{:3}", r + 1);
            for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns as i32) {
                if sheet.all_cells[r as usize][c as usize].is_error {
                    print!("{:^9}", "ERR");
                } else {
                    print!("{:^9}", sheet.all_cells[r as usize][c as usize].value);
                }
            }
            println!();
        }
    }
}

/*fn parse_cell_name(cell_name: &str) -> Option<(usize, usize)> {
    // Implement parsing logic based on your C code's parseCellName function
    unimplemented!()
}

fn parse_input(input: &str, sheet: &mut Sheet) {
    // Implement parsing logic based on your C code's parseInput function
    unimplemented!()
}

fn assign_cell(sheet: &mut Sheet, row: usize, col: usize, operation_id: i32, formula: &[Operand], count_operands: usize) {
    // Implement cell assignment logic based on your C code's assign_cell function
    unimplemented!()
}*/

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Out of bounds rows or columns");
        process::exit(1);
    }

    let rows: i32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Invalid row number");
        process::exit(1);
    });

    let columns: i32 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Invalid column number");
        process::exit(1);
    });

    if !(1..=MAX_ROWS).contains(&rows) || !(1..=MAX_COLS).contains(&columns) {
        eprintln!("Out of bounds rows or columns");
        process::exit(1);
    }

    let mut sheet = initialise(rows as i32, columns as i32);

    let mut input = String::new();
    let mut status_str = String::new();
    let mut execution_time = 0.0;
    let mut print_flag = true;
    display_sheet(&sheet);

    // let mut operation_id: i32 = 0;
    // let  mut edit_row: i32 = 0;
    // let mut edit_column: i32 = 0;
    // let mut count_operands: i32 = 0;
    // let mut formula: Vec<Operand> = vec![];

    loop {
        let mut operation_id: i32 = 0;
        let mut edit_row: i32 = 0;
        let mut edit_column: i32 = 0;
        let mut count_operands: i32 = 0;
        let mut formula: Vec<Operand> = vec![];
        unsafe {
            status_str = match STATUS {
                0 => "ok".to_string(),
                1 => {
                    STATUS = 0;
                    "Invalid Input".to_string()
                }
                2 => {
                    STATUS = 0;
                    "ok".to_string()
                }
                3 => {
                    STATUS = 0;
                    "cyclic dependence".to_string()
                }
                _ => "unknown status".to_string(),
            };

            print!("[{:.1}] ({}) > ", execution_time, status_str);
            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let start = Instant::now();

            match input {
                "q" => break,
                "disable_output" => print_flag = false,
                "enable_output" => print_flag = true,
                "w" =>if(CURRENT_ROW -10 < 0){CURRENT_ROW=0;}else{CURRENT_ROW = CURRENT_ROW - 10;},
                "s" => 
                    if(CURRENT_ROW+10 >rows){
                        CURRENT_ROW=CURRENT_ROW;
                    }
                    else if(CURRENT_ROW+20>rows){
                        CURRENT_ROW=rows-10;
                    }
                    else{
                        CURRENT_ROW=CURRENT_ROW+10;
                    },
                "a" =>  if(CURRENT_COL-10 < 0){
                            CURRENT_COL=0;
                        }
                        else{
                            CURRENT_COL=CURRENT_COL-10;
                        },
                "d" => if(CURRENT_COL+10 >columns){
                            CURRENT_COL=CURRENT_COL;
                        }
                        else if(CURRENT_COL+20>columns){
                            CURRENT_COL=columns-10;
                        }
                        else{
                            CURRENT_COL=CURRENT_COL+10;
                        },
                _ if input.starts_with("scroll_to ") => {
                    let mut new_row = 0;
                    let mut new_col = 0;
                    parse_cell_name(&input[10..], &mut new_row, &mut new_col);
                    if new_row < rows && new_col < columns {
                        CURRENT_ROW = new_row;
                        CURRENT_COL = new_col;
                        DISPLAY_ROW = 0;
                        DISPLAY_COLUMN = 0;
                    } else {
                        eprintln!("Out of bounds rows or columns");
                    }
                }
                _ => {
                    parse_input(
                        input,
                        &mut sheet,
                        rows,
                        columns,
                        &mut operation_id,
                        &mut edit_row,
                        &mut edit_column,
                        &mut count_operands,
                        &mut formula,
                    );
                    if STATUS != 1 {
                        assign_cell(&mut sheet, edit_row, edit_column, operation_id, formula);
                    }
                }
            }

            execution_time = start.elapsed().as_secs_f64();

            if print_flag {
                display_sheet(&sheet);
            }
        }
    }
}
