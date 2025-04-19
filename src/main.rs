use std::io::{self, Write};
use std::time::Instant;
use std::process;
use std::ffi::CString;
use std::ptr;
use std::cmp::{min, max};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// use rust_lab::cellsp::Operand;
// use rust_lab::cellsp::Spreadsheet;
use rust_lab::cell_extension::SpreadsheetExtension;
// use rust_lab::dependency_graph_final::assign_cell;
// use rust_lab::dependency_graph_final::initialise;
// use rust_lab::dependency_graph_final::STATUS;
use rust_lab::parser_visual_mode::parse_cell_name;
use rust_lab::parser_visual_mode::string_to_int;
// use rust_lab::input::parse_input;
use rust_lab::expression_utils::parse_formula;
use rust_lab::expression_parser::Expr;
use rust_lab::graph_extension::assign_cell_extension;
use rust_lab::graph_extension::initialise_extension;
use rust_lab::graph_extension::STATUS_extension;
use rust_lab::parser_visual_mode::parser_visual;
use rust_lab::read_mode::handle_read_command;


const MAX_ROWS: i32 = 999;
const MAX_COLS: i32 = 18278;
const VIEWPORT_SIZE: i32 = 10;

static mut CURRENT_ROW: i32 = 0;
static mut CURRENT_COL: i32 = 0;
static mut DISPLAY_CELL: i32 = 0;
static mut DISPLAY_ROW: i32 = 0;
static mut DISPLAY_COLUMN: i32 = 0;
static mut CURRENT_MODE: ModeOfSpreadsheet = ModeOfSpreadsheet::INSERT;



enum ModeOfSpreadsheet {
    NORMAL,
    INSERT,
    VISUAL,
    READ,
}




fn parser_normal(input: &str, rows: i32, cols: i32) {
    unsafe {
        let mut count = 1;
        let mut command = input;

        // Split the input into numeric prefix and the rest of the string
        let split_index = input.find(|c: char| !c.is_digit(10)).unwrap_or(0);
        let (num, cmd) = input.split_at(split_index);

        if !num.is_empty() {
            count = num.parse::<i32>().unwrap_or(1);
        }
        command = cmd;

        match command {
            "h" => CURRENT_COL = max(0, CURRENT_COL - count),
            "l" => CURRENT_COL = min(cols - 1, CURRENT_COL + count),
            "k" => CURRENT_ROW = max(0, CURRENT_ROW - count),
            "j" => CURRENT_ROW = min(rows - 1, CURRENT_ROW + count),
            "gg" => CURRENT_ROW = 0, // Go to the first row
            "G" => CURRENT_ROW = rows - 1, // Go to the last row
            _ => return,
        }
    }
}




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

fn display_sheet(sheet: &SpreadsheetExtension, shared_data: &Arc<Mutex<Vec<Vec<String>>>>) {
    let mut output: Vec<Vec<String>> = Vec::new();

    unsafe {
        let mut header_row = vec![String::from("")];
        for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns as i32) {
            header_row.push(format!("{:9}", get_col_label(c)));
        }
        output.push(header_row);

        for r in CURRENT_ROW..min(CURRENT_ROW + VIEWPORT_SIZE, sheet.rows as i32) {
            let mut row_vec = vec![format!("{:3}", r + 1)];
            for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns as i32) {
                let cell = &sheet.all_cells[r as usize][c as usize];
                if cell.is_error {
                    row_vec.push("ERR".to_string());
                } else {
                    row_vec.push(format!("{:9}", cell.value));
                }
            }
            output.push(row_vec);
        }
    }

    let mut data = shared_data.lock().unwrap();
    *data = output;
}


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

    let mut sheet = initialise_extension(rows as i32, columns as i32);


    let mut input = String::new();
    let mut status_str = String::new();
    let mut execution_time = 0.0;
    let mut print_flag = true;
    
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    rust_lab::display::launch_gui(shared_data.clone());
    display_sheet(&sheet, &shared_data);
    loop {
        // let mut operation_id: i32 = 0;
        // let mut edit_row: i32 = 0;
        // let mut edit_column: i32 = 0;
        // let mut count_operands: i32 = 0;
        // let mut formula: Vec<Operand> = vec![];
        unsafe {
            status_str = match STATUS_extension {
                0 => "ok".to_string(),
                1 => {
                    STATUS_extension = 0;
                    "Invalid Input".to_string()
                }
                2 => {
                    STATUS_extension = 0;
                    "ok".to_string()
                }
                3 => {
                    STATUS_extension = 0;
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
                "read" => CURRENT_MODE = ModeOfSpreadsheet::READ,
                "insert" => CURRENT_MODE = ModeOfSpreadsheet::INSERT,
                "normal" => CURRENT_MODE = ModeOfSpreadsheet::NORMAL,
                "visual" => CURRENT_MODE = ModeOfSpreadsheet::VISUAL,
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
                _ => 
                    match CURRENT_MODE {
                        ModeOfSpreadsheet::READ => {handle_read_command( input, &mut sheet);},

                        ModeOfSpreadsheet::NORMAL => {parser_normal(input, rows, columns);},

                        ModeOfSpreadsheet::INSERT => {
                            // Call parse_input and assign_cell_extension for INSERT mode
                            let parts: Vec<&str> = input.split('=').collect();
                            if parts.len() != 2 {
                                eprintln!("Invalid input format. Expected 'cell_name=expression'.");
                                STATUS_extension = 1;
                                continue;
                            }
                            let cell_name = parts[0].trim();
                            let expression = parts[1].trim();
                            let mut edit_r = 0;
                            let mut edit_col = 0;
                            parse_cell_name(cell_name, &mut edit_r, &mut edit_col);
    
                            match parse_formula(expression) {
                                Ok(parsed_formula) => {
                                    let expr = *parsed_formula;
                                    if STATUS_extension != 1 {
                                        assign_cell_extension(&mut sheet, edit_r, edit_col, expr);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error parsing formula: {}", e);
                                    STATUS_extension = 1;
                                }
                            }
                        }
                        ModeOfSpreadsheet::VISUAL => {
                            // Handle VISUAL mode commands (e.g., copy, paste)
                            parser_visual(input, &mut sheet);
                        }
                },
                    
            
            }

            execution_time = start.elapsed().as_secs_f64();

            if print_flag {
                display_sheet(&sheet, &shared_data);
            }
        }
    }
}