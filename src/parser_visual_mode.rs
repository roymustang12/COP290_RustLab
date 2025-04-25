use crate::cell_extension::SpreadsheetExtension;
use crate::expression_utils::parse_formula;
use crate::forecast::forecast;
use crate::graph_extension::STATUS_EXTENSION;
use crate::graph_extension::{UndoRedoStack, assign_cell_extension};
use crate::plot_graph::{plot_histogram, plot_line, plot_scatter};

/// Parses a cell name (e.g., "A1") into its row and column indices.
///
/// # Arguments
/// * `cell_name` - The cell name as a string (e.g., "A1").
/// * `row` - A mutable reference to store the parsed row index.
/// * `col` - A mutable reference to store the parsed column index.
pub fn parse_cell_name(cell_name: &str, row: &mut i32, col: &mut i32) {
    let chars: Vec<char> = cell_name.chars().collect();
    let mut i = 0;
    *col = 0;

    // Extract column part (letters)
    loop {
        if i >= chars.len() {
            break;
        }

        if chars[i] >= '0' && chars[i] <= '9' {
            break;
        } else if (chars[i].to_ascii_uppercase() as i32 - 'A' as i32) >= 26
            || (chars[i].to_ascii_uppercase() as i32 - 'A' as i32) < 0
        {
            unsafe {
                STATUS_EXTENSION = 1;
            }
            break;
        }

        *col = *col * 26 + (chars[i].to_ascii_uppercase() as i32 - 'A' as i32 + 1);
        i += 1;
    }
    *col -= 1;

    // Extract row part (numbers)
    let row_part = &cell_name[i..];
    *row = string_to_int(row_part) - 1;
}

/// Converts a numeric string to an integer.
///
/// # Arguments
/// * `num_str` - The numeric string to convert.
///
/// # Returns
/// The integer value of the string. Returns `0` if the string is invalid.
pub fn string_to_int(num_str: &str) -> i32 {
    let mut num = 0;
    let mut i = 0;
    let mut sign = 1;

    let chars: Vec<char> = num_str.chars().collect();

    if !chars.is_empty() {
        if chars[0] == '-' {
            sign = -1;
            i = 1;
        } else if chars[0] == '+' {
            i = 1;
        }
    }
    while i < chars.len() {
        let digit = chars[i] as i32 - '0' as i32;
        if !(0..=9).contains(&digit) {
            unsafe {
                STATUS_EXTENSION = 1;
            }
            break;
        }
        num = num * 10 + digit;
        i += 1;
    }

    num * sign
}

/// Parses and executes visual mode commands for the spreadsheet.
///
/// # Arguments
/// * `input` - The command string entered by the user.
/// * `sheet` - A mutable reference to the spreadsheet.
///
/// # Supported Commands
///
/// ## Filtering
/// * **Command**: `filter <range> <comparator> <value>`
/// * **Description**: Filters cells in the specified range based on the given condition.
/// * **Example**: `filter A1:B2 > 10`
///
/// ## Cutting a Single Cell
/// * **Command**: `dc <source_cell> <destination_cell>`
/// * **Description**: Cuts the value from the source cell and pastes it into the destination cell.
/// * **Example**: `dc A1 B1`
///
/// ## Copying a Single Cell
/// * **Command**: `yc <source_cell> <destination_cell>`
/// * **Description**: Copies the value from the source cell and pastes it into the destination cell.
/// * **Example**: `yc A1 B1`
///
/// ## Cutting a Range of Cells
/// * **Command**: `d <source_range> <destination_range>`
/// * **Description**: Cuts the values from the source range and pastes them into the destination range.
/// * **Example**: `d A1:B2 C1:D2`
///
/// /// ## Copying a Range of Cells
/// * **Command**: `y <source_range> <destination_range>`
/// * **Description**: Copies the values from the source range and pastes them into the destination range.
/// * **Example**: `y A1:B2 C1:D2`
///
/// ## Plotting a Histogram
/// * **Command**: `plot_histogram <range> <filename>`
/// * **Description**: Generates a histogram for the values in the specified range and saves it to the given file.
/// * **Example**: `plot_histogram A1:A10 histogram.png`
///
/// ## Plotting a Line Graph
/// * **Command**: `plot_line <range> <filename>`
/// * **Description**: Generates a line graph for the values in the specified range and saves it to the given file.
/// * **Example**: `plot_line A1:A10 line.png`
///
/// ## Plotting a Scatter Plot
/// * **Command**: `plot_scatter <x_range> <y_range> <filename>`
/// * **Description**: Generates a scatter plot for the values in the specified x and y ranges and saves it to the given file.
/// * **Example**: `plot_scatter A1:A10 B1:B10 scatter.png`
///
/// ## Forecasting Future Values
/// * **Command**: `forecast <length> <x_range> <y_range> <filename>`
/// * **Description**: Forecasts future values based on the given x and y ranges using linear regression and saves the scatter plot to the given file.
/// * **Example**: `forecast 5 A1:A10 B1:B10 forecast.png`
///   /// ## Applying Bold Formatting
/// * **Command**: `b <cell_name>`
/// * **Description**: Applies bold formatting to the specified cell.
/// * **Example**: `b A1`
///
/// ## Applying Italics Formatting
/// * **Command**: `i <cell_name>`
/// * **Description**: Applies italics formatting to the specified cell.
/// * **Example**: `i A1`
///
/// # Behavior
/// Executes commands such as filtering, copying, cutting, pasting, plotting, and forecasting.
pub fn parser_visual(
    input: &str,
    sheet: &mut SpreadsheetExtension,
    undo_manager: &mut UndoRedoStack,
) {
    unsafe {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            STATUS_EXTENSION = 1;
            return;
        }

        match parts[0] {
            "b" => {
                if parts.len() != 2 {
                    STATUS_EXTENSION = 1;
                    return;
                }
                let mut row = 0;
                let mut col = 0;
                parse_cell_name(parts[1], &mut row, &mut col);

                if row < 0
                    || col < 0
                    || row >= sheet.rows
                    || col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                sheet.all_cells[row as usize][col as usize].is_bold = true;
                //     println!("Applied bold formatting to cell {}", parts[1]);
            }

            "i" => {
                if parts.len() != 2 {
                    STATUS_EXTENSION = 1;
                    return;
                }
                let mut row = 0;
                let mut col = 0;
                parse_cell_name(parts[1], &mut row, &mut col);

                if row < 0
                    || col < 0
                    || row >= sheet.rows
                    || col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                sheet.all_cells[row as usize][col as usize].is_italics = true;
                // println!("Applied italics formatting to cell {}", parts[1]);
            }
            "filter" => {
                if parts.len() != 4 {
                    eprintln!("Invalid format. Expected: filter <range> <comparator> <value>");
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                // Parse range
                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut start_row,
                    &mut start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut end_row,
                    &mut end_col,
                );

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Extract comparator and value
                let comparator = parts[2];
                let value: f64 = match parts[3].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Invalid value for filter condition");
                        STATUS_EXTENSION = 1;
                        return;
                    }
                };

                // Vector to store filtered results
                let mut filtered_cells: Vec<(String, f64)> = Vec::new();

                // Traverse the range and apply the filter
                for r in start_row..=end_row {
                    for c in start_col..=end_col {
                        let cell_value = sheet.all_cells[r as usize][c as usize].value as f64;

                        // Apply the comparator
                        let condition_met = match comparator {
                            ">" => cell_value > value,
                            "<" => cell_value < value,
                            ">=" => cell_value >= value,
                            "<=" => cell_value <= value,
                            "=" => cell_value == value,
                            _ => {
                                eprintln!("Invalid comparator. Expected one of: >, <, >=, <=, =");
                                STATUS_EXTENSION = 1;
                                return;
                            }
                        };

                        // If the condition is met, add the cell to the result
                        if condition_met {
                            let cell_name = format!(
                                "{}{}",
                                (b'A' + c as u8) as char, // Convert column index to letter
                                r + 1                     // Convert row index to 1-based
                            );
                            filtered_cells.push((cell_name, cell_value));
                        }
                    }
                }

                // Print the filtered results
                if filtered_cells.is_empty() {
                    println!("No cells match the filter condition");
                } else {
                    println!("Filtered cells:");
                    for (cell_name, cell_value) in filtered_cells {
                        println!("{}: {}", cell_name, cell_value);
                    }
                }
            }
            "dc" => {
                //cut cell
                if parts.len() != 3 {
                    STATUS_EXTENSION = 1;
                    return;
                }
                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut value = sheet.all_cells[start_row as usize][start_col as usize]
                    .value
                    .to_string();
                if sheet.all_cells[start_row as usize][start_col as usize].is_error {
                    value = "1/0".to_string();
                }
                let value2 = "0";
                if let Ok(parsed_formula) = parse_formula(value2) {
                    assign_cell_extension(
                        sheet,
                        undo_manager,
                        start_row,
                        start_col,
                        *parsed_formula,
                    );
                }
                if let Ok(parsed_formula) = parse_formula(&value) {
                    assign_cell_extension(sheet, undo_manager, end_row, end_col, *parsed_formula);
                }
            }
            "yc" => {
                //copy cell
                if parts.len() != 3 {
                    STATUS_EXTENSION = 1;
                    return;
                }
                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;
                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut value = sheet.all_cells[start_row as usize][start_col as usize]
                    .value
                    .to_string();
                if sheet.all_cells[start_row as usize][start_col as usize].is_error {
                    value = "1/0".to_string();
                }

                if let Ok(parsed_formula) = parse_formula(&value) {
                    assign_cell_extension(sheet, undo_manager, end_row, end_col, *parsed_formula);
                }
            }
            "d" => {
                // cut and paste fro a range of cells
                if parts.len() != 3 {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;
                let mut dest_start_row = 0;
                let mut dest_start_col = 0;
                let mut dest_end_row = 0;
                let mut dest_end_col = 0;

                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut start_row,
                    &mut start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut end_row,
                    &mut end_col,
                );

                parse_cell_name(
                    parts[2].split(':').next().unwrap(),
                    &mut dest_start_row,
                    &mut dest_start_col,
                );
                parse_cell_name(
                    parts[2].split(':').nth(1).unwrap(),
                    &mut dest_end_row,
                    &mut dest_end_col,
                );

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if dest_start_row < 0
                    || dest_start_col < 0
                    || dest_start_row >= sheet.rows
                    || dest_start_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if dest_end_row < 0
                    || dest_end_col < 0
                    || dest_end_row >= sheet.rows
                    || dest_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Ensure the source and destination ranges have the same dimensions
                if (end_row - start_row) != (dest_end_row - dest_start_row)
                    || (end_col - start_col) != (dest_end_col - dest_start_col)
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Cut (move) the values/formulas
                for r in 0..=(end_row - start_row) {
                    for c in 0..=(end_col - start_col) {
                        let src_row = start_row + r;
                        let src_col = start_col + c;
                        let dest_row = dest_start_row + r;
                        let dest_col = dest_start_col + c;

                        // Get the value/formula from the source cell
                        let mut value = sheet.all_cells[src_row as usize][src_col as usize]
                            .value
                            .to_string();
                        if sheet.all_cells[start_row as usize][start_col as usize].is_error {
                            value = "1/0".to_string();
                        }

                        // Clear the source cell
                        let clear_formula = "0";
                        if let Ok(parsed_formula) = parse_formula(clear_formula) {
                            assign_cell_extension(
                                sheet,
                                undo_manager,
                                src_row,
                                src_col,
                                *parsed_formula,
                            );
                        }

                        // Assign the value/formula to the destination cell
                        if let Ok(parsed_formula) = parse_formula(&value) {
                            assign_cell_extension(
                                sheet,
                                undo_manager,
                                dest_row,
                                dest_col,
                                *parsed_formula,
                            );
                        }
                    }
                }
            }
            "y" => {
                if parts.len() != 3 {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;
                let mut dest_start_row = 0;
                let mut dest_start_col = 0;
                let mut dest_end_row = 0;
                let mut dest_end_col = 0;

                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut start_row,
                    &mut start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut end_row,
                    &mut end_col,
                );

                parse_cell_name(
                    parts[2].split(':').next().unwrap(),
                    &mut dest_start_row,
                    &mut dest_start_col,
                );
                parse_cell_name(
                    parts[2].split(':').nth(1).unwrap(),
                    &mut dest_end_row,
                    &mut dest_end_col,
                );

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if dest_start_row < 0
                    || dest_start_col < 0
                    || dest_start_row >= sheet.rows
                    || dest_start_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if dest_end_row < 0
                    || dest_end_col < 0
                    || dest_end_row >= sheet.rows
                    || dest_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Ensure the source and destination ranges have the same dimensions
                if (end_row - start_row) != (dest_end_row - dest_start_row)
                    || (end_col - start_col) != (dest_end_col - dest_start_col)
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Copy the values/formulas
                for r in 0..=(end_row - start_row) {
                    for c in 0..=(end_col - start_col) {
                        let src_row = start_row + r;
                        let src_col = start_col + c;
                        let dest_row = dest_start_row + r;
                        let dest_col = dest_start_col + c;

                        // Get the value/formula from the source cell
                        let mut value = sheet.all_cells[src_row as usize][src_col as usize]
                            .value
                            .to_string();
                        if sheet.all_cells[start_row as usize][start_col as usize].is_error {
                            value = "1/0".to_string();
                        }

                        // Assign the value/formula to the destination cell
                        if let Ok(parsed_formula) = parse_formula(&value) {
                            assign_cell_extension(
                                sheet,
                                undo_manager,
                                dest_row,
                                dest_col,
                                *parsed_formula,
                            );
                        }
                    }
                }
            }
            "plot_histogram" => {
                if parts.len() != 3 {
                    eprintln!(
                        "Invalid format for histogram plot. Expected: plot_histogram <range> <filename>"
                    );
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                // Parse range
                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut start_row,
                    &mut start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut end_row,
                    &mut end_col,
                );

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Extract data
                let mut data = Vec::new();
                for r in start_row..=end_row {
                    for c in start_col..=end_col {
                        let value = sheet.all_cells[r as usize][c as usize].value as f64;
                        data.push(value);
                    }
                }

                // Call the histogram plot function
                if let Err(e) = plot_histogram(&data, parts[2]) {
                    eprintln!("Error generating histogram: {}", e);
                }
            }

            "plot_line" => {
                if parts.len() != 3 {
                    eprintln!("Invalid format. Expected: plot_line <range> <filename>");
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                // Parse the start and end cell names from the range
                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut start_row,
                    &mut start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut end_row,
                    &mut end_col,
                );

                if start_row < 0
                    || start_col < 0
                    || start_row >= sheet.rows
                    || start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if end_row < 0 || end_col < 0 || end_row >= sheet.rows || end_col >= sheet.columns {
                    STATUS_EXTENSION = 1;
                    return;
                }

                // Initialize a Vec for each column
                let num_cols = end_col - start_col + 1;
                let mut data: Vec<Vec<f64>> = vec![Vec::new(); num_cols as usize];

                // Populate the data for each column within the specified range
                for r in start_row..=end_row {
                    for (i, c) in (start_col..=end_col).enumerate() {
                        let value = sheet.all_cells[r as usize][c as usize].value as f64;
                        data[i].push(value); // Group by column
                    }
                }

                // Pass `start_row` to plot_line to correctly reflect the row numbers on the x-axis
                if let Err(e) = plot_line(&data, parts[2]) {
                    eprintln!("Error generating line plot: {}", e);
                }
            }

            "plot_scatter" => {
                if parts.len() != 4 {
                    eprintln!(
                        "Invalid format. Expected: plot_scatter <x_range> <y_range> <filename>"
                    );
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut x_start_row = 0;
                let mut x_start_col = 0;
                let mut x_end_row = 0;
                let mut x_end_col = 0;

                let mut y_start_row = 0;
                let mut y_start_col = 0;
                let mut y_end_row = 0;
                let mut y_end_col = 0;

                parse_cell_name(
                    parts[1].split(':').next().unwrap(),
                    &mut x_start_row,
                    &mut x_start_col,
                );
                parse_cell_name(
                    parts[1].split(':').nth(1).unwrap(),
                    &mut x_end_row,
                    &mut x_end_col,
                );

                parse_cell_name(
                    parts[2].split(':').next().unwrap(),
                    &mut y_start_row,
                    &mut y_start_col,
                );
                parse_cell_name(
                    parts[2].split(':').nth(1).unwrap(),
                    &mut y_end_row,
                    &mut y_end_col,
                );

                if x_start_row < 0
                    || x_start_col < 0
                    || x_start_row >= sheet.rows
                    || x_start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if x_end_row < 0
                    || x_end_col < 0
                    || x_end_row >= sheet.rows
                    || x_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if y_start_row < 0
                    || y_start_col < 0
                    || y_start_row >= sheet.rows
                    || y_start_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if y_end_row < 0
                    || y_end_col < 0
                    || y_end_row >= sheet.rows
                    || y_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut x_data = Vec::new();
                let mut y_data = Vec::new();

                for r in x_start_row..=x_end_row {
                    for c in x_start_col..=x_end_col {
                        x_data.push(sheet.all_cells[r as usize][c as usize].value as f64);
                    }
                }

                for r in y_start_row..=y_end_row {
                    for c in y_start_col..=y_end_col {
                        y_data.push(sheet.all_cells[r as usize][c as usize].value as f64);
                    }
                }

                if x_data.len() != y_data.len() {
                    eprintln!("x and y data must be the same length");
                    STATUS_EXTENSION = 1;
                    return;
                }

                if let Err(e) = plot_scatter(&x_data, &y_data, parts[3]) {
                    eprintln!("Error generating scatter plot: {}", e);
                }
            }

            "forecast" => {
                if parts.len() != 5 {
                    eprintln!(
                        "Invalid format. Expected: plot_scatter <x_range> <y_range> <filename>"
                    );
                    STATUS_EXTENSION = 1;
                    return;
                }

                println!("enterde into foreacst");
                let forecast_len = string_to_int(parts[1]);

                let mut x_start_row = 0;
                let mut x_start_col = 0;
                let mut x_end_row = 0;
                let mut x_end_col = 0;

                let mut y_start_row = 0;
                let mut y_start_col = 0;
                let mut y_end_row = 0;
                let mut y_end_col = 0;

                parse_cell_name(
                    parts[2].split(':').next().unwrap(),
                    &mut x_start_row,
                    &mut x_start_col,
                );
                parse_cell_name(
                    parts[2].split(':').nth(1).unwrap(),
                    &mut x_end_row,
                    &mut x_end_col,
                );

                parse_cell_name(
                    parts[3].split(':').next().unwrap(),
                    &mut y_start_row,
                    &mut y_start_col,
                );
                parse_cell_name(
                    parts[3].split(':').nth(1).unwrap(),
                    &mut y_end_row,
                    &mut y_end_col,
                );

                if x_start_row < 0
                    || x_start_col < 0
                    || x_start_row >= sheet.rows
                    || x_start_col >= sheet.columns
                    || STATUS_EXTENSION == 1
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if x_end_row < 0
                    || x_end_col < 0
                    || x_end_row >= sheet.rows
                    || x_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if y_start_row < 0
                    || y_start_col < 0
                    || y_start_row >= sheet.rows
                    || y_start_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                if y_end_row < 0
                    || y_end_col < 0
                    || y_end_row >= sheet.rows
                    || y_end_col >= sheet.columns
                {
                    STATUS_EXTENSION = 1;
                    return;
                }

                let mut x_data = Vec::new();
                let mut y_data = Vec::new();

                for r in x_start_row..=x_end_row {
                    for c in x_start_col..=x_end_col {
                        x_data.push(sheet.all_cells[r as usize][c as usize].value as f64);
                    }
                }

                for r in y_start_row..=y_end_row {
                    for c in y_start_col..=y_end_col {
                        y_data.push(sheet.all_cells[r as usize][c as usize].value as f64);
                    }
                }

                let temp_x = x_data.clone();
                let temp_y = y_data.clone();
                let common_diff = x_data[1] - x_data[0];
                let mut next_x = x_data[x_data.len() - 1];

                for i in 0..forecast_len {
                    println!("Iteration: {}", i);
                    next_x += common_diff;
                    let next_y = ((forecast(next_x, &temp_x, &temp_y)) as i32) as f64;
                    x_data.push(next_x);
                    y_data.push(next_y);
                }

                println!("entered into second stage");

                if x_data.len() != y_data.len() {
                    println!("x and y data must be the same length");
                    STATUS_EXTENSION = 1;
                    return;
                }

                println!("{:?}", x_data);
                println!("{:?}", y_data);

                if let Err(e) = plot_scatter(&x_data, &y_data, parts[4]) {
                    println!("chud gaye");
                    println!("Error generating scatter plot: {}", e);
                }
            }

            _ => {
                STATUS_EXTENSION = 1;
            }
        }
    }
}
