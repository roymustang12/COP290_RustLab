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
use rust_lab::graph_extension::{assign_cell_extension, UndoRedoStack,perform_undo,perform_redo};
use rust_lab::graph_extension::initialise_extension;
use rust_lab::graph_extension::STATUS_extension;
use rust_lab::parser_visual_mode::parser_visual;
use rust_lab::read_mode::handle_read_command;


const MAX_ROWS: i32 = 999;
const MAX_COLS: i32 = 18278;
const VIEWPORT_SIZE: i32 = 10;

// static mut CURRENT_ROW: i32 = 0;
// static mut CURRENT_COL: i32 = 0;
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




fn parser_normal(input: &str, rows: i32, cols: i32, current_row: &Arc<Mutex<usize>>, current_col: &Arc<Mutex<usize>>) {
    let mut count: i32 = 1;
    let split_index = input.find(|c: char| !c.is_digit(10)).unwrap_or(0);
    let (num, cmd) = input.split_at(split_index);

    if !num.is_empty() {
        count = num.parse::<i32>().unwrap_or(1);
    }

    match cmd {
        "h" => {
            let mut col = current_col.lock().unwrap();
            *col = max(0, *col as i32 - count) as usize;
        }
        "l" => {
            let mut col = current_col.lock().unwrap();
            *col = min(cols as i32 - 1, *col as i32 + count) as usize;
        }
        "k" => {
            let mut row = current_row.lock().unwrap();
            *row = max(0, *row as i32 - count) as usize;
        }
        "j" => {
            let mut row = current_row.lock().unwrap();
            *row = min(rows as i32 - 1, *row as i32  + count) as usize;
        }
        "gg" => *current_row.lock().unwrap() = 0,
        "G" => *current_row.lock().unwrap() = rows as usize  - 1,
        _ => {}
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

fn display_sheet(sheet: &SpreadsheetExtension, shared_data: &Arc<Mutex<Vec<Vec<String>>>>, current_row: &Arc<Mutex<usize>>,current_col: &Arc<Mutex<usize>>) {
    let mut output: Vec<Vec<String>> = Vec::new();
    let mut dr = current_row.lock().unwrap();
    let mut dc = current_col.lock().unwrap();


        let mut header_row = vec![String::from("")];
        for c in *dc..min(*dc + 15, sheet.columns as usize) {
            header_row.push(format!("{:9}", get_col_label(c as i32)));
        }
        output.push(header_row);

        for r in *dr..min(*dr + 24, sheet.rows as usize) {
            let mut row_vec = vec![format!("{:3}", r + 1)];
            for c in *dc..min(*dc + 15, sheet.columns as usize) {
                let cell = &sheet.all_cells[r as usize][c as usize];
                let mut cell_display = format!("{:9}", cell.value);

                if cell.is_bold && cell.is_italics {
                    cell_display = format!("~{}~", cell_display); // Underline
                }
                else if cell.is_bold {
                    cell_display = format!("*{}*", cell_display); // Bold
                }
                else if cell.is_italics {
                    cell_display = format!("_{}_", cell_display); // Italics
                }
            

                if cell.is_error {
                    row_vec.push("ERR".to_string());
                } else {
                    row_vec.push(cell_display);
                }
            }
            output.push(row_vec);
        
    }

    let mut data = shared_data.lock().unwrap();
    *data = output;
}

fn main() {

    // let current_row = Arc::new(Mutex::new(0));
    // let current_col = Arc::new(Mutex::new(0));
    // let display_row = Arc::new(Mutex::new(0));
    // let display_col = Arc::new(Mutex::new(0));


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

    let sheet = Arc::new(Mutex::new(initialise_extension(rows, columns))); // Wrap the sheet
    let undo_manager = Arc::new(Mutex::new(UndoRedoStack::new())); // Wrap the undo_manager in Arc<Mutex<>>

    let shared_data = Arc::new(Mutex::new(vec![vec![String::new(); columns as usize]; rows as usize]));
    let input_text = Arc::new(Mutex::new(String::new()));
    let status_extension = Arc::new(Mutex::new(0)); // Shared status variable
    let current_row = Arc::new(Mutex::new(0)); // Starting row of the visible portion
    let current_col = Arc::new(Mutex::new(0)); // Starting column of the visible portion

    
    // Launch the GUI on the main thread
    rust_lab::display::launch_gui(
        shared_data.clone(), // Pass the shared data
        sheet.clone(), // Pass the sheet
        input_text.clone(),
        status_extension.clone(),
        current_row.clone(),
        current_col.clone(),
        undo_manager.clone(), // Pass the undo_manager
    );
    display_sheet( &sheet.lock().unwrap(), &shared_data,&current_row,&current_col);

    // Process input in a separate thread
    let input_processing_thread = {
        let shared_data = shared_data.clone();
        let input_text = input_text.clone();
        let status_extension = status_extension.clone();
        std::thread::spawn(move || {
            let mut status_str = String::new();
            let mut execution_time = 0.0;
            let mut print_flag = true;

            loop {
              

                let mut input = input_text.lock().unwrap();
                if input.is_empty() {
                    // If no input, release the lock and sleep briefly to avoid busy-waiting
                    display_sheet(&sheet.lock().unwrap(), &shared_data,&current_row,&current_col);
                    drop(input);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }

                let command = input.clone();
                *input = String::new(); // Clear the input after reading
                drop(input); // Release the lock

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

                    let start = Instant::now();

                    match command.as_str() {
                        "q" => break,
                        "undo" => {perform_undo(&mut sheet.lock().unwrap(),&mut undo_manager.lock().unwrap());},
                        "redo" =>{ perform_redo(&mut sheet.lock().unwrap(),&mut undo_manager.lock().unwrap());},
                        "read" => CURRENT_MODE = ModeOfSpreadsheet::READ,
                        "insert" => CURRENT_MODE = ModeOfSpreadsheet::INSERT,
                        "normal" => CURRENT_MODE = ModeOfSpreadsheet::NORMAL,
                        "visual" => CURRENT_MODE = ModeOfSpreadsheet::VISUAL,
                        "w" => {
                                let mut dr = current_row.lock().unwrap();
                                *dr = if (*dr as i32 - 24 < 0) { 0 } else { *dr - 24 };
                            }
                            "s" => {
                                let mut dr = current_row.lock().unwrap();
                                *dr = if *dr + 24 > rows as usize{
                                    *dr // No change if exceeding
                                } else if *dr + 48 > rows as usize {
                                    rows as usize  - 24
                                } else {
                                    *dr + 24
                                };
                            }
                            "a" => {
                                let mut dc = current_col.lock().unwrap();
                                *dc = if *dc as i32 - 15 < 0 { 0 } else { *dc - 15 };
                            }
                            "d" => {
                                let mut dc = current_col.lock().unwrap();
                                *dc = if *dc + 15 > columns as usize {
                                    *dc // No change if exceeding
                                } else if *dc + 30 > columns as usize {
                                    columns as usize  - 15
                                } else {
                                    *dc + 15
                                };
                            }
                        "disable_output" => print_flag = false,
                        "enable_output" => print_flag = true,
                        _ if command.starts_with("scroll_to ") => {
                            let mut new_row = 0;
                            let mut new_col = 0;
                            parse_cell_name(&command[10..], &mut new_row, &mut new_col);
                            if new_row < rows && new_col < columns {
                                let mut dr = current_row.lock().unwrap();
                                *dr = new_row as usize;
                                let mut dc = current_col.lock().unwrap();
                                *dc =new_col as usize;
                                DISPLAY_ROW = 0;
                                DISPLAY_COLUMN = 0;
                            } else {
                                eprintln!("Out of bounds rows or columns");
                                STATUS_extension = 1;
                            }
                        }
                        _ => match CURRENT_MODE {
                            ModeOfSpreadsheet::READ => {handle_read_command(&command,&mut sheet.lock().unwrap(), &mut undo_manager.lock().unwrap());},
                            ModeOfSpreadsheet::NORMAL => {parser_normal(&command, rows, columns, &current_row, &current_col);},
                            ModeOfSpreadsheet::INSERT => {
                                let parts: Vec<&str> = command.split('=').collect();
                                if parts.len() != 2 {
                                    // eprintln!("Invalid input format. Expected 'cell_name=expression'.");
                                    STATUS_extension = 1;
                                    let mut status = status_extension.lock().unwrap();
                                    *status = STATUS_extension; 
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
                                            assign_cell_extension(&mut sheet.lock().unwrap(),&mut undo_manager.lock().unwrap() ,edit_r, edit_col, expr);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error parsing formula: {}", e);
                                        STATUS_extension = 1;
                                    }
                                }
                            }
                            ModeOfSpreadsheet::VISUAL => parser_visual(&command, &mut sheet.lock().unwrap(), &mut undo_manager.lock().unwrap()),
                        },
                    }

                    let mut status = status_extension.lock().unwrap();
                    *status = STATUS_extension; 

                   
                    execution_time = start.elapsed().as_secs_f64();

                    if print_flag {
                        display_sheet(&sheet.lock().unwrap(), &shared_data,&current_row,&current_col);
                          
                    }
                }
            }
        })
    };

    // Wait for the input processing thread to finish
    input_processing_thread.join().unwrap();
}