use crate::cellsp2::{CellReference, Operand};

/// Checks if a string contains any alphabetic characters.
///
/// # Arguments
///
/// * `s` - A string slice to check.
///
/// # Returns
///
/// `true` if the string contains any alphabetic characters, otherwise `false`.
fn contains_alphabet(s: &str) -> bool {
    s.chars().any(|c| c.is_alphabetic())
}

/// Converts a string representation of an integer to an `i32`.
///
/// # Arguments
///
/// * `num_str` - A string slice representing the integer.
///
/// # Returns
///
/// The integer value of the string. If the string is invalid, it sets a global status flag.
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
                crate::dependency_graph_final::STATUS = 1;
            }
            break;
        }
        num = num * 10 + digit;
        i += 1;
    }

    num * sign
}

/// Counts the occurrences of a specific character in a string.
///
/// # Arguments
///
/// * `ch` - The character to count.
/// * `s` - The string slice to search.
///
/// # Returns
///
/// The number of times the character appears in the string.
fn count_occurrences(ch: char, s: &str) -> i32 {
    s.chars().filter(|&c| c == ch).count() as i32
}

/// Determines if a string is an arithmetic expression.
///
/// # Arguments
///
/// * `expression` - A string slice representing the expression.
///
/// # Returns
///
/// `true` if the string contains arithmetic operators (`+`, `-`, `*`, `/`), otherwise `false`.
fn is_arithmetic_expression(expression: &str) -> bool {
    expression.contains('+')
        || expression.contains('-')
        || expression.contains('*')
        || expression.contains('/')
}

/// Parses a cell name into its row and column indices.
///
/// # Arguments
///
/// * `cell_name` - The cell name as a string slice (e.g., "A1").
/// * `row` - A mutable reference to store the row index.
/// * `col` - A mutable reference to store the column index.
///
/// # Notes
///
/// If the cell name is invalid, it sets a global status flag.
pub fn parse_cell_name_1(cell_name: &str, row: &mut i32, col: &mut i32) {
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
                crate::dependency_graph_final::STATUS = 1;
            }
            break;
        }

        *col = *col * 26 + (chars[i].to_ascii_uppercase() as i32 - 'A' as i32 + 1);
        i += 1;
    }
    *col -= 1;

    // Extract row part (numbers)
    let row_part = &cell_name[i..];
    if !contains_alphabet(row_part) && !is_arithmetic_expression(row_part) {
        *row = string_to_int(row_part) - 1;
    } else {
        unsafe {
            crate::dependency_graph_final::STATUS = 1;
        }
    }
}

/// Checks if a string represents a function.
///
/// # Arguments
///
/// * `expression` - A string slice representing the expression.
///
/// # Returns
///
/// `true` if the string contains a function name (e.g., "MIN(", "MAX("), otherwise `false`.
fn is_function(expression: &str) -> bool {
    expression.contains("MIN(")
        || expression.contains("MAX(")
        || expression.contains("AVG(")
        || expression.contains("SUM(")
        || expression.contains("STDEV(")
        || expression.contains("SLEEP(")
}

/// Assigns a numeric value to an arithmetic operator.
///
/// # Arguments
///
/// * `op` - The arithmetic operator (`+`, `-`, `*`, `/`).
///
/// # Returns
///
/// A numeric value corresponding to the operator. If the operator is invalid, it sets a global status flag.
fn assign_value(op: char) -> i32 {
    match op {
        '+' => 3,
        '-' => 4,
        '*' => 5,
        '/' => 6,
        _ => {
            unsafe {
                crate::dependency_graph_final::STATUS = 1;
            }
            0
        }
    }
}

/// Parses an input string and updates the spreadsheet accordingly.
///
/// # Arguments
///
/// * `input` - The input string to parse.
/// * `_spreadsheet` - A mutable reference to the spreadsheet.
/// * `rows` - The number of rows in the spreadsheet.
/// * `cols` - The number of columns in the spreadsheet.
/// * `operation_id` - A mutable reference to store the operation ID.
/// * `edit_row` - A mutable reference to store the row index of the edited cell.
/// * `edit_column` - A mutable reference to store the column index of the edited cell.
/// * `count_operands` - A mutable reference to store the number of operands.
/// * `formula` - A mutable reference to store the parsed formula.
///
/// # Notes
///
/// If the input is invalid, it sets a global status flag.
#[allow(clippy::too_many_arguments)]
pub fn parse_input(
    input: &str,
    _spreadsheet: &mut crate::cellsp2::Spreadsheet,
    rows: i32,
    cols: i32,
    operation_id: &mut i32,
    edit_row: &mut i32,
    edit_column: &mut i32,
    count_operands: &mut i32,
    formula: &mut Vec<Operand>,
) {
    let parts: Vec<&str> = input.split('=').collect();
    if parts.len() != 2 {
        unsafe {
            crate::dependency_graph_final::STATUS = 1;
        }
        return;
    }
    // println!("  i was here");
    let cell_name = parts[0].trim();
    let expression = parts[1].trim();
    let mut edit_r = 0;
    let mut edit_col = 0;
    parse_cell_name_1(cell_name, &mut edit_r, &mut edit_col);
    if edit_r < 0 || edit_r >= rows || edit_col < 0 || edit_col >= cols {
        unsafe {
            crate::dependency_graph_final::STATUS = 1;
        }
        return;
    }
    *edit_row = edit_r;
    *edit_column = edit_col;

    if expression
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c == '-' || c == '+')
        || (expression.chars().next().is_some_and(|c| c.is_alphabetic())
            && !expression.contains('('))
    {
        if !contains_alphabet(expression) && !is_arithmetic_expression(&expression[1..]) {
            *count_operands = 1;
            let formula_vec = vec![Operand::Constant(string_to_int(expression))];
            *formula = formula_vec;
            *operation_id = 1;
        } else if contains_alphabet(expression) && !is_arithmetic_expression(&expression[1..]) {
            let mut row1 = 0;
            let mut col1 = 0;
            parse_cell_name_1(expression, &mut row1, &mut col1);
            if row1 < 0 || row1 >= rows || col1 < 0 || col1 >= cols {
                unsafe {
                    crate::dependency_graph_final::STATUS = 1;
                }
                return;
            }
            *count_operands = 1;
            let formula_vec = vec![Operand::CellOperand(CellReference {
                row: row1,
                column: col1,
            })];
            *formula = formula_vec;
            *operation_id = 2;
        } else if (contains_alphabet(expression) && is_arithmetic_expression(&expression[1..]))
            || (!contains_alphabet(expression) && is_arithmetic_expression(&expression[1..]))
        {
            *count_operands = 2;
            let mut formula_vec = Vec::with_capacity(2);

            if !(expression.starts_with('+') || expression.starts_with('-')) {
                let mut parts = expression.splitn(2, |c| ['+', '-', '*', '/'].contains(&c));
                let operand1 = parts.next().unwrap_or("");

                if let Some(rest) = parts.next() {
                    let op = expression.chars().nth(operand1.len()).unwrap_or('+');
                    let operand2 = rest;

                    if operand2.is_empty() {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    if operand1.chars().next().is_some_and(|c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name_1(operand1, &mut r1, &mut c1);

                        if r1 < 0 || r1 > rows || c1 < 0 || c1 > cols {
                            unsafe { crate::dependency_graph_final::STATUS = 1 }
                            return;
                        }

                        formula_vec.push(Operand::CellOperand(CellReference {
                            row: r1,
                            column: c1,
                        }));
                        *operation_id = assign_value(op);
                    } else {
                        formula_vec.push(Operand::Constant(string_to_int(operand1)));
                        *operation_id = assign_value(op);
                    }

                    if operand2.chars().next().is_some_and(|c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name_1(operand2, &mut r1, &mut c1);

                        if r1 < 0 || r1 > rows || c1 < 0 || c1 > cols {
                            unsafe {
                                crate::dependency_graph_final::STATUS = 1;
                            }
                            return;
                        }

                        formula_vec.push(Operand::CellOperand(CellReference {
                            row: r1,
                            column: c1,
                        }));
                    } else {
                        formula_vec.push(Operand::Constant(string_to_int(operand2)));
                    }
                } else {
                    unsafe {
                        crate::dependency_graph_final::STATUS = 1;
                    }
                }
                *formula = formula_vec;
            } else if expression.starts_with('+') || expression.starts_with('-') {
                // Handle expressions starting with ++... or --..
                if expression
                    .chars()
                    .nth(1)
                    .is_some_and(|c| c == '+' || c == '-')
                {
                    unsafe {
                        crate::dependency_graph_final::STATUS = 1;
                    }
                    return;
                }

                let sign = expression.chars().next().unwrap();
                let expr_without_sign = &expression[1..];

                let mut parts = expr_without_sign.splitn(2, |c| ['+', '-', '*', '/'].contains(&c));
                let operand1 = parts.next().unwrap_or("");

                if let Some(rest) = parts.next() {
                    let op = expr_without_sign.chars().nth(operand1.len()).unwrap_or('+');
                    let operand2 = rest;

                    if operand1.chars().next().is_some_and(|c| c.is_alphabetic()) {
                        // // yahan konsa case handle ho raha tha ? like agar +A1 OPERAND .... HAI TO NEGATIVE KYUN NI CHECK KIYA ? +A1 valid tha ya nahin check akrna hai
                        // let mut r1 = 0;
                        // let mut c1 = 0;
                        // parse_cell_name_1(operand1, &mut r1, &mut c1);

                        // if r1 < 0 || r1 > rows || c1 < 0 || c1 > cols {
                        //     unsafe {dependency_graph_final::STATUS = 1;}
                        //     return;
                        // }

                        // formula_vec.push(Operand {
                        //     type_flag: 1,
                        //     operand_value: OperandValue::CellOperand(spreadsheet.all_cells[r1 as usize][c1 as usize].clone())
                        // });
                        // unsafe {*operation_id = assign_value(op);}
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    } else {
                        let mut value = string_to_int(operand1);
                        if sign == '-' {
                            value *= -1;
                        }

                        formula_vec.push(Operand::Constant(value));
                        *operation_id = assign_value(op);
                    }

                    if operand2.chars().next().is_some_and(|c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name_1(operand2, &mut r1, &mut c1);

                        if r1 < 0 || r1 > rows || c1 < 0 || c1 > cols {
                            unsafe {
                                crate::dependency_graph_final::STATUS = 1;
                            }
                            return;
                        }

                        formula_vec.push(Operand::CellOperand(CellReference {
                            row: r1,
                            column: c1,
                        }));
                    } else {
                        formula_vec.push(Operand::Constant(string_to_int(operand2)));
                    }
                } else {
                    unsafe {
                        crate::dependency_graph_final::STATUS = 1;
                    }
                }
                *formula = formula_vec;
            }
        } else {
            unsafe {
                crate::dependency_graph_final::STATUS = 1;
            }
            return;
        }
    } else if is_function(expression) {
        // println!("i was here");
        let open_paren_pos = expression.find('(');
        let close_paren_pos = expression.rfind(')');

        if open_paren_pos.is_none()
            || close_paren_pos.is_none()
            || open_paren_pos.unwrap() > close_paren_pos.unwrap() - 1
        {
            unsafe {
                crate::dependency_graph_final::STATUS = 1;
            }
            // println!("i entered heer 1");
        }

        let open_count = count_occurrences('(', expression);
        let close_count = count_occurrences(')', expression);

        if open_count != close_count {
            unsafe {
                crate::dependency_graph_final::STATUS = 1;
            }
            //    println!("i entered heer 2");
        }

        if let (Some(open_pos), Some(close_pos)) = (open_paren_pos, close_paren_pos) {
            let function_name = &expression[0..open_pos];
            let range = &expression[open_pos + 1..close_pos];

            unsafe {
                // println!("i was here");
                *operation_id = match function_name {
                    "MIN" => 7,
                    "MAX" => 8,
                    "AVG" => 9,
                    "SUM" => 10,
                    "STDEV" => 11,
                    "SLEEP" => 12,
                    _ => {
                        crate::dependency_graph_final::STATUS = 1;
                        0
                    }
                };
            }

            if function_name == "SLEEP" {
                *count_operands = 1;

                let mut formula_vec = Vec::with_capacity(1);

                if contains_alphabet(range) {
                    let mut r3 = 0;
                    let mut c3 = 0;
                    parse_cell_name_1(range, &mut r3, &mut c3);

                    if r3 < 0 || r3 > rows || c3 < 0 || c3 > cols {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    formula_vec.push(Operand::CellOperand(CellReference {
                        row: r3,
                        column: c3,
                    }));
                } else {
                    formula_vec.push(Operand::Constant(string_to_int(range)));
                }

                *formula = formula_vec;
            } else {
                // Handle range functions (MIN, MAX, AVG, SUM, STDEV)
                let range_parts: Vec<&str> = range.split(':').collect();

                if range_parts.len() == 2 {
                    let start = range_parts[0];
                    let end = range_parts[1];

                    let mut r3 = 0;
                    let mut c3 = 0;
                    let mut r4 = 0;
                    let mut c4 = 0;

                    parse_cell_name_1(start, &mut r3, &mut c3);

                    if r3 < 0 || r3 > rows || c3 < 0 || c3 > cols {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    parse_cell_name_1(end, &mut r4, &mut c4);
                    if r4 < 0 || r4 > rows || c4 < 0 || c4 > cols {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    if r3 > r4 || c3 > c4 {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    let numbers = (r4 - r3 + 1) * (c4 - c3 + 1);

                    *count_operands = (r4 - r3 + 1) * (c4 - c3 + 1);
                    let mut formula_vec = Vec::with_capacity(numbers as usize);

                    for i in r3..=r4 {
                        for j in c3..=c4 {
                            formula_vec
                                .push(Operand::CellOperand(CellReference { row: i, column: j }));
                        }
                    }

                    *formula = formula_vec;
                } else {
                    unsafe {
                        crate::dependency_graph_final::STATUS = 1;
                    }
                    // println!("i was here 4");
                }
            }
        } else {
            unsafe {
                crate::dependency_graph_final::STATUS = 1;
            }
            // println!("i was here 5");
        }
    } else {
        unsafe {
            crate::dependency_graph_final::STATUS = 1;
        }
        // println!("i was here 6");
    }
}
