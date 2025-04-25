#[cfg(feature = "main2")]
use std::cmp::max;
use std::cmp::min;
#[cfg(feature = "main1")]
use std::io::{self, Write};
use std::process;
#[cfg(feature = "main1")]
use std::time::Instant;

// #[cfg(feature = "main2")]
// use rust_lab::cell_extension::SpreadsheetExtension;
#[cfg(feature = "main2")]
use rust_lab::expression_utils::parse_formula;
#[cfg(feature = "main2")]
use rust_lab::graph_extension::STATUS_EXTENSION;
#[cfg(feature = "main2")]
use rust_lab::graph_extension::initialise_extension;
#[cfg(feature = "main2")]
use rust_lab::graph_extension::{UndoRedoStack, assign_cell_extension, perform_redo, perform_undo};
#[cfg(feature = "main2")]
use rust_lab::parser_visual_mode::parse_cell_name;
#[cfg(feature = "main2")]
use rust_lab::parser_visual_mode::parser_visual;
#[cfg(feature = "main2")]
use rust_lab::read_mode::handle_read_command;
#[cfg(feature = "main2")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "main1")]
use rust_lab::cellsp2::Operand;
#[cfg(feature = "main1")]
use rust_lab::cellsp2::Spreadsheet;
#[cfg(feature = "main1")]
use rust_lab::dependency_graph_final::STATUS;
#[cfg(feature = "main1")]
use rust_lab::dependency_graph_final::assign_cell;
#[cfg(feature = "main1")]
use rust_lab::dependency_graph_final::initialise;
#[cfg(feature = "main1")]
use rust_lab::input::parse_cell_name_1;
#[cfg(feature = "main1")]
use rust_lab::input::parse_input;

const MAX_ROWS: i32 = 999;
const MAX_COLS: i32 = 18278;
#[cfg(feature = "main1")]
const VIEWPORT_SIZE: i32 = 10;
#[cfg(feature = "main1")]
static mut CURRENT_ROW: i32 = 0;
#[cfg(feature = "main1")]
static mut CURRENT_COL: i32 = 0;
// static mut DISPLAY_CELL: i32 = 0;
static mut DISPLAY_ROW: i32 = 0;
static mut DISPLAY_COLUMN: i32 = 0;
#[cfg(feature = "main2")]
static mut CURRENT_MODE: ModeOfSpreadsheet = ModeOfSpreadsheet::Insert;

#[cfg(feature = "main2")]
enum ModeOfSpreadsheet {
    Normal,
    Insert,
    Visual,
    Read,
}

/// Parses a command in normal mode for `main2`.Using this we traverse using h,j,k,l,g,gg commands through the spraedsheet.
///
/// # Arguments
///
/// * `input` - The command input string.
/// * `rows` - The total number of rows in the spreadsheet.
/// * `cols` - The total number of columns in the spreadsheet.
/// * `current_row` - The current row index of the viewport.
/// * `current_col` - The current column index of the viewport.
#[cfg(feature = "main2")]
fn parser_normal(
    input: &str,
    rows: i32,
    cols: i32,
    current_row: &Arc<Mutex<usize>>,
    current_col: &Arc<Mutex<usize>>,
) {
    let mut count: i32 = 1;
    let split_index = input.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
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
            *col = min(cols - 15, *col as i32 + count) as usize; // Ensure we don't go beyond the last viewport
        }
        "k" => {
            let mut row = current_row.lock().unwrap();
            *row = max(0, *row as i32 - count) as usize;
        }
        "j" => {
            let mut row = current_row.lock().unwrap();
            *row = min(rows - 24, *row as i32 + count) as usize; // Ensure we don't go beyond the last viewport
        }
        "gg" => *current_row.lock().unwrap() = 0,
        "G" => *current_row.lock().unwrap() = max(0, rows as usize - 24), // Adjust for viewport size
        _ => {}
    }
}

/// Displays the spreadsheet in the terminal for `main1`.
///
/// # Arguments
///
/// * `sheet` - A reference to the `Spreadsheet` to display.
#[cfg(feature = "main1")]
fn display_sheet1(sheet: &Spreadsheet) {
    unsafe {
        print!("   ");
        for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns) {
            print!("{:^9}", get_col_label(c));
        }
        println!();

        for r in CURRENT_ROW..min(CURRENT_ROW + VIEWPORT_SIZE, sheet.rows) {
            print!("{:3}", r + 1);
            for c in CURRENT_COL..min(CURRENT_COL + VIEWPORT_SIZE, sheet.columns) {
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

/// Generates a column label (e.g., "A", "B", ..., "Z", "AA") for a given column index.
///
/// # Arguments
///
/// * `col` - The column index (0-based).
///
/// # Returns
///
/// A string representing the column label.
#[cfg(feature = "main1")]
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

/// Handles the main functionality for `main2`i.e the one with complex adn more integrated fetaures .
///
/// This function initializes a spreadsheet, launches a GUI, and processes user input
/// in a separate thread. It supports advanced features like undo/redo, visual modes,
/// and formula parsing.
///
/// # Arguments
///
/// * `args` - Command-line arguments specifying the number of rows and columns.
///
/// # Commands
///
/// * `q` - Quit the program.
/// * `undo` - Undo the last operation.
/// * `redo` - Redo the last undone operation.
/// * `read` - Switch to read mode.
/// * `insert` - Switch to insert mode.
/// * `normal` - Switch to normal mode.
/// * `visual` - Switch to visual mode.
/// * `w` - Scroll up.
/// * `s` - Scroll down.
/// * `a` - Scroll left.
/// * `d` - Scroll right.
/// * `scroll_to <cell>` - Scroll to a specific cell.
/// * `<cell_name>=<expression>` - Assign a formula or value to a cell.
#[cfg(feature = "main2")]
fn main_functionality2() {
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

    let shared_data = Arc::new(Mutex::new(vec![
        vec![String::new(); columns as usize];
        rows as usize
    ]));
    let input_text = Arc::new(Mutex::new(String::new()));
    let status_extension = Arc::new(Mutex::new(0)); // Shared status variable
    let current_row = Arc::new(Mutex::new(0)); // Starting row of the visible portion
    let current_col = Arc::new(Mutex::new(0)); // Starting column of the visible portion

    // Launch the GUI on the main thread
    rust_lab::display::launch_gui(
        shared_data.clone(), // Pass the shared data
        sheet.clone(),       // Pass the sheet
        input_text.clone(),
        status_extension.clone(),
        current_row.clone(),
        current_col.clone(),
        undo_manager.clone(), // Pass the undo_manager
    );
    // display_sheet( &sheet.lock().unwrap(), &shared_data,&current_row,&current_col);

    // Process input in a separate thread
    let input_processing_thread = {
        let _shared_data = shared_data.clone();
        let input_text = input_text.clone();
        let status_extension = status_extension.clone();
        std::thread::spawn(move || {
            // let mut status_str = String::new();
            // let mut execution_time = 0.0;
            // let mut print_flag = true;

            loop {
                let mut input = input_text.lock().unwrap();
                if input.is_empty() {
                    // If no input, release the lock and sleep briefly to avoid busy-waiting
                    // display_sheet(&sheet.lock().unwrap(), &shared_data,&current_row,&current_col);
                    drop(input);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }

                let command = input.clone();
                *input = String::new(); // Clear the input after reading
                drop(input); // Release the lock

                unsafe {
                    // status_str = match STATUS_EXTENSION {
                    //     0 => "ok".to_string(),
                    //     1 => {
                    //         STATUS_EXTENSION = 0;
                    //         "Invalid Input".to_string()
                    //     }
                    //     2 => {
                    //         STATUS_EXTENSION = 0;
                    //         "ok".to_string()
                    //     }
                    //     3 => {
                    //         STATUS_EXTENSION = 0;
                    //         "cyclic dependence".to_string()
                    //     }
                    //     _ => "unknown status".to_string(),
                    // };

                    // let start = Instant::now();

                    match command.as_str() {
                        "q" => break,
                        "undo" => {
                            perform_undo(
                                &mut sheet.lock().unwrap(),
                                &mut undo_manager.lock().unwrap(),
                            );
                        }
                        "redo" => {
                            perform_redo(
                                &mut sheet.lock().unwrap(),
                                &mut undo_manager.lock().unwrap(),
                            );
                        }
                        "read" => CURRENT_MODE = ModeOfSpreadsheet::Read,
                        "insert" => CURRENT_MODE = ModeOfSpreadsheet::Insert,
                        "normal" => CURRENT_MODE = ModeOfSpreadsheet::Normal,
                        "visual" => CURRENT_MODE = ModeOfSpreadsheet::Visual,
                        "w" => {
                            let mut dr = current_row.lock().unwrap();
                            *dr = if *dr as i32 - 24 < 0 { 0 } else { *dr - 24 };
                        }
                        "s" => {
                            let mut dr = current_row.lock().unwrap();
                            *dr = if *dr + 24 > rows as usize {
                                *dr // No change if exceeding
                            } else if *dr + 48 > rows as usize {
                                rows as usize - 24
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
                                columns as usize - 15
                            } else {
                                *dc + 15
                            };
                        }
                        _ if command.starts_with("scroll_to ") => {
                            let mut new_row = 0;
                            let mut new_col = 0;
                            parse_cell_name(&command[10..], &mut new_row, &mut new_col);
                            if new_row < rows && new_col < columns {
                                let mut dr = current_row.lock().unwrap();
                                *dr = new_row as usize;
                                let mut dc = current_col.lock().unwrap();
                                *dc = new_col as usize;
                                DISPLAY_ROW = 0;
                                DISPLAY_COLUMN = 0;
                            } else {
                                eprintln!("Out of bounds rows or columns");
                                STATUS_EXTENSION = 1;
                            }
                        }
                        _ => match CURRENT_MODE {
                            ModeOfSpreadsheet::Read => {
                                handle_read_command(
                                    &command,
                                    &mut sheet.lock().unwrap(),
                                    &mut undo_manager.lock().unwrap(),
                                );
                            }
                            ModeOfSpreadsheet::Normal => {
                                parser_normal(&command, rows, columns, &current_row, &current_col);
                            }
                            ModeOfSpreadsheet::Insert => {
                                let parts: Vec<&str> = command.split('=').collect();
                                if parts.len() != 2 {
                                    // eprintln!("Invalid input format. Expected 'cell_name=expression'.");
                                    STATUS_EXTENSION = 1;
                                    let mut status = status_extension.lock().unwrap();
                                    *status = STATUS_EXTENSION;
                                    continue;
                                }
                                let cell_name = parts[0].trim();
                                let expression = parts[1].trim();
                                let mut edit_r = 0;
                                let mut edit_col = 0;
                                parse_cell_name(cell_name, &mut edit_r, &mut edit_col);

                                if edit_r < 0
                                    || edit_col < 0
                                    || edit_r >= rows
                                    || edit_col >= columns
                                    || STATUS_EXTENSION == 1
                                {
                                    STATUS_EXTENSION = 1;
                                    continue;
                                }

                                match parse_formula(expression) {
                                    Ok(parsed_formula) => {
                                        let expr = *parsed_formula;
                                        if STATUS_EXTENSION != 1 {
                                            assign_cell_extension(
                                                &mut sheet.lock().unwrap(),
                                                &mut undo_manager.lock().unwrap(),
                                                edit_r,
                                                edit_col,
                                                expr,
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error parsing formula: {}", e);
                                        STATUS_EXTENSION = 1;
                                    }
                                }
                            }
                            ModeOfSpreadsheet::Visual => parser_visual(
                                &command,
                                &mut sheet.lock().unwrap(),
                                &mut undo_manager.lock().unwrap(),
                            ),
                        },
                    }

                    let mut status = status_extension.lock().unwrap();
                    *status = STATUS_EXTENSION;

                    // execution_time = start.elapsed().as_secs_f64();

                    // if print_flag {
                    //     display_sheet(&sheet.lock().unwrap(), &shared_data,&current_row,&current_col);

                    // }
                }
            }
        })
    };

    // Wait for the input processing thread to finish
    input_processing_thread.join().unwrap();
}

/// Handles the main functionality for `main1`i.e the one wiht basic faetures .
///
/// This function initializes a spreadsheet, processes user input, and updates the spreadsheet
/// based on the input commands.
///
/// # Arguments
///
/// * `args` - Command-line arguments specifying the number of rows and columns.
///
/// # Commands
///
/// * `q` - Quit the program.
/// * `w` - Scroll up.
/// * `s` - Scroll down.
/// * `a` - Scroll left.
/// * `d` - Scroll right.
/// * `scroll_to <cell>` - Scroll to a specific cell.
/// * `<cell_name>=<expression>` - Assign a formula or value to a cell.
#[cfg(feature = "main1")]
fn main_functionality1() {
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

    let mut sheet = initialise(rows, columns);

    let mut input = String::new();

    #[allow(unused_assignments)]
    let mut status_str = String::new();

    let mut execution_time = 0.0;
    let mut print_flag = true;
    display_sheet1(&sheet);

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
                "w" => {
                    if CURRENT_ROW - 10 < 0 {
                        CURRENT_ROW = 0;
                    } else {
                        CURRENT_ROW -= 10;
                    }
                }
                "s" => {
                    if CURRENT_ROW + 10 > rows {
                    } else if CURRENT_ROW + 20 > rows {
                        CURRENT_ROW = rows - 10;
                    } else {
                        CURRENT_ROW += 10;
                    }
                }
                "a" => {
                    if CURRENT_COL - 10 < 0 {
                        CURRENT_COL = 0;
                    } else {
                        CURRENT_COL -= 10;
                    }
                }
                "d" => {
                    if CURRENT_COL + 10 > columns {
                    } else if CURRENT_COL + 20 > columns {
                        CURRENT_COL = columns - 10;
                    } else {
                        CURRENT_COL += 10;
                    }
                }
                _ if input.starts_with("scroll_to ") => {
                    let mut new_row = 0;
                    let mut new_col = 0;
                    parse_cell_name_1(&input[10..], &mut new_row, &mut new_col);
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
                display_sheet1(&sheet);
            }
        }
    }
}

/// The main entry point of the program.
///
/// This function conditionally calls `main_functionality1` or `main_functionality2`
/// based on the enabled feature (`main1` or `main2`).
fn main() {
    #[cfg(feature = "main1")]
    main_functionality1();

    #[cfg(feature = "main2")]
    main_functionality2();
}
