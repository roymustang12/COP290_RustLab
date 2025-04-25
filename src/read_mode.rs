use crate::cell_extension::SpreadsheetExtension;
use crate::expression_utils::parse_formula;
use crate::graph_extension::{UndoRedoStack, assign_cell_extension};
use std::error::Error;
use std::fs::File;
use std::path::Path;

/// Reads a CSV file and populates the spreadsheet with its contents.
///
/// # Arguments
/// * `filename` - The name of the CSV file to read.
/// * `sheet` - A mutable reference to the spreadsheet.
///
/// # Returns
/// * `Ok(())` if the file is successfully read and processed.
/// * `Err` if an error occurs (e.g., file not found or invalid CSV format).
///
/// # Behavior
/// * Trims whitespace from cell values.
/// * Supports flexible row lengths in the CSV file.
/// * Handles formula cells (starting with `=`) by parsing and assigning them.
///
pub fn read_csv_file(
    filename: &str,
    sheet: &mut SpreadsheetExtension,
    undo_manager: &mut UndoRedoStack,
) -> Result<(), Box<dyn Error>> {
    let path = Path::new(filename);

    if !path.exists() {
        return Err(format!("File not found: {}", filename).into());
    }

    let file = File::open(path)?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false) // No headers in CSV
        .flexible(true) // Allow rows with different lengths
        .trim(csv::Trim::All) // Trim whitespace
        .from_reader(file);

    let mut row_num = 0;
    for result in rdr.records() {
        let record = result?;

        // Don't exceed spreadsheet bounds
        if row_num >= sheet.rows as usize {
            println!("Warning: CSV file has more rows than the spreadsheet can accommodate");
            break;
        }

        // Process each cell in the row
        for (col_num, cell_value) in record.iter().enumerate() {
            // Don't exceed spreadsheet bounds
            if col_num >= sheet.columns as usize {
                println!("Warning: CSV file has more columns than the spreadsheet can accommodate");
                break;
            }

            // Process cell value
            if let Some(formula_str) = cell_value.strip_prefix("=") {
                // Handle formula cells
                match parse_formula(formula_str) {
                    Ok(expr) => {
                        assign_cell_extension(
                            sheet,
                            undo_manager,
                            row_num as i32,
                            col_num as i32,
                            *expr,
                        );
                    }
                    Err(_) => {
                        // Invalid formula, treat as text or number
                        store_cell_value(sheet, row_num, col_num, cell_value);
                    }
                }
            } else {
                // Handle non-formula cells
                match parse_formula(cell_value) {
                    Ok(expr) => {
                        assign_cell_extension(
                            sheet,
                            undo_manager,
                            row_num as i32,
                            col_num as i32,
                            *expr,
                        );
                    }
                    Err(_) => {
                        // Invalid formula, treat as text or number
                        store_cell_value(sheet, row_num, col_num, cell_value);
                    }
                }
            }
        }

        row_num += 1;
    }

    println!("Successfully loaded {} rows from {}", row_num, filename);
    Ok(())
}

/// Stores a cell value in the spreadsheet.
///
/// # Arguments
/// * `sheet` - A mutable reference to the spreadsheet.
/// * `row` - The row index of the cell.
/// * `col` - The column index of the cell.
/// * `value` - The value to store in the cell.
///
/// # Behavior
/// * If the value is numeric, it is stored as an integer.
/// * If the value is non-numeric, it is stored as `0` (or as a string if supported).
///
fn store_cell_value(sheet: &mut SpreadsheetExtension, row: usize, col: usize, value: &str) {
    // Try to parse as integer
    if let Ok(num_value) = value.parse::<i32>() {
        sheet.all_cells[row][col].value = num_value;
        sheet.all_cells[row][col].is_error = false;
    } else {
        // Non-numeric value - store as 0 or implement string storage
        sheet.all_cells[row][col].value = 0;
        // If your spreadsheet supports string values, you would store it here
    }
}

/// Handles the `read` command to load a CSV file into the spreadsheet.
///
/// # Arguments
/// * `cmd` - The command string (e.g., `read data.csv`).
/// * `sheet` - A mutable reference to the spreadsheet.
///
/// # Returns
/// * `true` if the command is successfully executed.
/// * `false` if the command is invalid or an error occurs.
///
pub fn handle_read_command(
    cmd: &str,
    sheet: &mut SpreadsheetExtension,
    undo_manager: &mut UndoRedoStack,
) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.len() != 2 || parts[0] != "read" {
        return false;
    }

    let filename = parts[1];
    match read_csv_file(filename, sheet, undo_manager) {
        Ok(_) => true,
        Err(e) => {
            println!("Error reading CSV file: {}", e);
            false
        }
    }
}
