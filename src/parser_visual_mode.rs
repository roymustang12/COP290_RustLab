use crate::graph_extension::assign_cell_extension;
use crate::cell_extension::SpreadsheetExtension;
use crate::graph_extension::STATUS_extension;
use crate::expression_utils::parse_formula;
use crate::plot_graph::{plot_histogram, plot_line, plot_scatter};



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
                STATUS_extension= 1;
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
        if digit < 0 || digit > 9 {
            unsafe {
                STATUS_extension = 1;
            }
            break;
        }
        num = num * 10 + digit;
        i += 1;
    }

    num * sign
}



pub fn parser_visual(input: &str, sheet: &mut SpreadsheetExtension) {
    unsafe {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            STATUS_extension= 1;
            return;
        }

        match parts[0] {
            "filter" => {
                        if parts.len() != 4 {
                            eprintln!("Invalid format. Expected: filter <range> <comparator> <value>");
                            STATUS_extension = 1;
                            return;
                        }

                        let mut start_row = 0;
                        let mut start_col = 0;
                        let mut end_row = 0;
                        let mut end_col = 0;

                        // Parse range
                        parse_cell_name(parts[1].split(':').next().unwrap(), &mut start_row, &mut start_col);
                        parse_cell_name(parts[1].split(':').nth(1).unwrap(), &mut end_row, &mut end_col);

                        // Extract comparator and value
                        let comparator = parts[2];
                        let value: f64 = match parts[3].parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("Invalid value for filter condition");
                                STATUS_extension = 1;
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
                                        STATUS_extension = 1;
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
            "dc" => {         //cut cell
                if parts.len() != 3 {
                    STATUS_extension= 1;
                    return;
                }
                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;
                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                let value = sheet.all_cells[start_row as usize][start_col as usize].value.to_string();
                let value2 = "0";
                if let Ok(parsed_formula) = parse_formula(&value2) {
                    assign_cell_extension(sheet, start_row, start_col, *parsed_formula);
                }
                
                if let Ok(parsed_formula) = parse_formula(&value) {
                    assign_cell_extension(sheet, end_row, end_col, *parsed_formula);
                }
            }
            "yc" => { //copy cell
                if parts.len() != 3 {
                    STATUS_extension= 1;
                    return;
                }
                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;
                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                
                let value = sheet.all_cells[start_row as usize][start_col as usize].value.to_string();

                if let Ok(parsed_formula) = parse_formula(&value) {
                    assign_cell_extension(sheet, end_row, end_col, *parsed_formula);
                }

               
            } 
            "d" => {   // cut and paste fro a range of cells
                if parts.len() != 5 {
                    STATUS_extension= 1;
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

                // Parse source range
                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                // Parse destination range
                parse_cell_name(parts[3], &mut dest_start_row, &mut dest_start_col);
                parse_cell_name(parts[4], &mut dest_end_row, &mut dest_end_col);

                // Ensure the source and destination ranges have the same dimensions
                if (end_row - start_row) != (dest_end_row - dest_start_row)
                    || (end_col - start_col) != (dest_end_col - dest_start_col)
                {
                    STATUS_extension= 1;
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
                        let value = sheet.all_cells[src_row as usize][src_col as usize].value.to_string();

                        // Clear the source cell
                        let clear_formula = "0";
                        if let Ok(parsed_formula) = parse_formula(clear_formula) {
                            assign_cell_extension(sheet, src_row, src_col, *parsed_formula);
                        }

                        // Assign the value/formula to the destination cell
                        if let Ok(parsed_formula) = parse_formula(&value) {
                            assign_cell_extension(sheet, dest_row, dest_col, *parsed_formula);
                        }
                    }
                }

            }
            "y" => {

                if parts.len() != 5 {
                    STATUS_extension= 1;
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

                // Parse source range
                parse_cell_name(parts[1], &mut start_row, &mut start_col);
                parse_cell_name(parts[2], &mut end_row, &mut end_col);

                // Parse destination range
                parse_cell_name(parts[3], &mut dest_start_row, &mut dest_start_col);
                parse_cell_name(parts[4], &mut dest_end_row, &mut dest_end_col);

                // Ensure the source and destination ranges have the same dimensions
                if (end_row - start_row) != (dest_end_row - dest_start_row)
                    || (end_col - start_col) != (dest_end_col - dest_start_col)
                {
                    STATUS_extension= 1;
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
                        let value = sheet.all_cells[src_row as usize][src_col as usize].value.to_string();

                        // Assign the value/formula to the destination cell
                        if let Ok(parsed_formula) = parse_formula(&value) {
                            assign_cell_extension(sheet, dest_row, dest_col, *parsed_formula);
                        }
                    }
                }

            }
           "plot_histogram" => {
                if parts.len() != 3 {
                    eprintln!("Invalid format for histogram plot. Expected: plot_histogram <range> <filename>");
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                // Parse range
                parse_cell_name(parts[1].split(':').next().unwrap(), &mut start_row, &mut start_col);
                parse_cell_name(parts[1].split(':').nth(1).unwrap(), &mut end_row, &mut end_col);

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
                    STATUS_extension = 1;
                    return;
                }

                let mut start_row = 0;
                let mut start_col = 0;
                let mut end_row = 0;
                let mut end_col = 0;

                parse_cell_name(parts[1].split(':').next().unwrap(), &mut start_row, &mut start_col);
                parse_cell_name(parts[1].split(':').nth(1).unwrap(), &mut end_row, &mut end_col);

                let mut data = Vec::new();
                for r in start_row..=end_row {
                    for c in start_col..=end_col {
                        let value = sheet.all_cells[r as usize][c as usize].value as f64;
                        data.push(value);
                    }
                }

                if let Err(e) = plot_line(&data, parts[2]) {
                    eprintln!("Error generating line plot: {}", e);
                }
            }

            "plot_scatter" => {
                if parts.len() != 4 {
                    eprintln!("Invalid format. Expected: plot_scatter <x_range> <y_range> <filename>");
                    STATUS_extension = 1;
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

                parse_cell_name(parts[1].split(':').next().unwrap(), &mut x_start_row, &mut x_start_col);
                parse_cell_name(parts[1].split(':').nth(1).unwrap(), &mut x_end_row, &mut x_end_col);

                parse_cell_name(parts[2].split(':').next().unwrap(), &mut y_start_row, &mut y_start_col);
                parse_cell_name(parts[2].split(':').nth(1).unwrap(), &mut y_end_row, &mut y_end_col);

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
                    STATUS_extension = 1;
                    return;
                }

                if let Err(e) = plot_scatter(&x_data, &y_data, parts[3]) {
                    eprintln!("Error generating scatter plot: {}", e);
                }
            }

            _ => {
                STATUS_extension = 1;
            }
        }
    }
}