use crate::cell_extension::SpreadsheetExtension;
use crate::expression_utils::parse_formula;
use crate::graph_extension::assign_cell_extension;
use std::error::Error;
use std::fs::File;
use std::path::Path;


pub fn read_csv_file(filename: &str, sheet: &mut SpreadsheetExtension) -> Result<(), Box<dyn Error>> {
    let path = Path::new(filename);
    
    
    if !path.exists() {
        return Err(format!("File not found: {}", filename).into());
    }
    
    
    let file = File::open(path)?;
    
    
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)  // Allow rows with different lengths
        .trim(csv::Trim::All)  // Trim whitespace
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
            if cell_value.starts_with("=") {
                // Handle formula cells
                match parse_formula(&cell_value[1..]) {
                    Ok(expr) => {
                        
                       assign_cell_extension(sheet, row_num as i32, col_num as i32, *expr);
                    },
                    Err(_) => {
                        // Invalid formula, treat as text or number
                        store_cell_value(sheet, row_num, col_num, cell_value);
                    }
                }
            } else {
                // Handle non-formula cells
                store_cell_value(sheet, row_num, col_num, cell_value);
            }
        }
        
        row_num += 1;
    }
    
    println!("Successfully loaded {} rows from {}", row_num, filename);
    Ok(())
}

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




pub fn handle_read_command(cmd: &str, sheet: &mut SpreadsheetExtension) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    
    if parts.len() != 2 || parts[0] != "read" {
        return false;
    }
    
    let filename = parts[1];
    match read_csv_file(filename, sheet) {
        Ok(_) => true,
        Err(e) => {
            println!("Error reading CSV file: {}", e);
            false
        }
    }
}