use crate::cellsp::{CellReference, Operand};

// #[derive(Clone)]
// struct Cell{
//  value: i32,
// }

// struct Sheet{
//     all_cells: Vec<Vec<Cell>>,
// }

// #[derive(Clone)]
// enum OperandValue{
//     Constant(i32),
//     CellOperand(Cell),
// }

// #[derive(Clone)]
// struct Operand{
//     type_flag: i32,
//     operand_value: OperandValue,
// }

// Global variables
// static mut dependency_graph_final::STATUS: i32 = 0;

fn contains_alphabet(s: &str) -> bool {
    s.chars().any(|c| c.is_alphabetic())
}

fn string_to_int(num_str: &str) -> i32 {
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
                crate::dependency_graph_final::STATUS = 1;
            }
            break;
        }
        num = num * 10 + digit;
        i += 1;
    }

    num * sign
}

// Count occurrences of a character in a string
fn count_occurrences(ch: char, s: &str) -> i32 {
    s.chars().filter(|&c| c == ch).count() as i32
}

fn is_arithmetic_expression(expression: &str) -> bool {
    expression.contains('+')
        || expression.contains('-')
        || expression.contains('*')
        || expression.contains('/')
}

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

fn is_function(expression: &str) -> bool {
    expression.contains("MIN(")
        || expression.contains("MAX(")
        || expression.contains("AVG(")
        || expression.contains("SUM(")
        || expression.contains("STDEV(")
        || expression.contains("SLEEP(")
}

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

pub fn parse_input(
    input: &str,
    spreadsheet: &mut crate::cellsp::Spreadsheet,
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
    parse_cell_name(cell_name, &mut edit_r, &mut edit_col);
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
        .map_or(false, |c| c.is_digit(10) || c == '-' || c == '+')
        || (expression
            .chars()
            .next()
            .map_or(false, |c| c.is_alphabetic())
            && !expression.contains('('))
    {
        if !contains_alphabet(expression) && !is_arithmetic_expression(&expression[1..]) {
            *count_operands = 1;
            let mut formula_vec = Vec::with_capacity(1);
            formula_vec.push(Operand::Constant(string_to_int(expression)));
            *formula = formula_vec;
            *operation_id = 1;
        } else if contains_alphabet(expression) && !is_arithmetic_expression(&expression[1..]) {
            let mut row1 = 0;
            let mut col1 = 0;
            parse_cell_name(&expression, &mut row1, &mut col1);
            if row1 < 0 || row1 >= rows || col1 < 0 || col1 >= cols {
                unsafe {
                    crate::dependency_graph_final::STATUS = 1;
                }
                return;
            }
            *count_operands = 1;
            let mut formula_vec = Vec::with_capacity(1);
            formula_vec.push(Operand::CellOperand(CellReference {
                row: row1,
                column: col1,
            }));
            *formula = formula_vec;
            *operation_id = 2;
        } else if (contains_alphabet(expression) && is_arithmetic_expression(&expression[1..]))
            || (!contains_alphabet(expression) && is_arithmetic_expression(&expression[1..]))
        {
            *count_operands = 2;
            let mut formula_vec = Vec::with_capacity(2);

            if !(expression.starts_with('+') || expression.starts_with('-')) {
                let mut parts =
                    expression.splitn(2, |c| c == '+' || c == '-' || c == '*' || c == '/');
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

                    if operand1.chars().next().map_or(false, |c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name(operand1, &mut r1, &mut c1);

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

                    if operand2.chars().next().map_or(false, |c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name(operand2, &mut r1, &mut c1);

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
                    .map_or(false, |c| c == '+' || c == '-')
                {
                    unsafe {
                        crate::dependency_graph_final::STATUS = 1;
                    }
                    return;
                }

                let sign = expression.chars().next().unwrap();
                let expr_without_sign = &expression[1..];

                let mut parts =
                    expr_without_sign.splitn(2, |c| c == '+' || c == '-' || c == '*' || c == '/');
                let operand1 = parts.next().unwrap_or("");

                if let Some(rest) = parts.next() {
                    let op = expr_without_sign.chars().nth(operand1.len()).unwrap_or('+');
                    let operand2 = rest;

                    if operand1.chars().next().map_or(false, |c| c.is_alphabetic()) {
                        // // yahan konsa case handle ho raha tha ? like agar +A1 OPERAND .... HAI TO NEGATIVE KYUN NI CHECK KIYA ? +A1 valid tha ya nahin check akrna hai
                        // let mut r1 = 0;
                        // let mut c1 = 0;
                        // parse_cell_name(operand1, &mut r1, &mut c1);

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

                    if operand2.chars().next().map_or(false, |c| c.is_alphabetic()) {
                        let mut r1 = 0;
                        let mut c1 = 0;
                        parse_cell_name(operand2, &mut r1, &mut c1);

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
                    parse_cell_name(range, &mut r3, &mut c3);

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

                    parse_cell_name(start, &mut r3, &mut c3);

                    if r3 < 0 || r3 > rows || c3 < 0 || c3 > cols {
                        unsafe {
                            crate::dependency_graph_final::STATUS = 1;
                        }
                        return;
                    }

                    parse_cell_name(end, &mut r4, &mut c4);
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

// fn main() {
//     // Initialize a sample spreadsheet with 3x3 cells
//     let mut spreadsheet = Sheet {
//         all_cells: vec![
//             vec![Cell { value: 1 }, Cell { value: 2 }, Cell { value: 3 }],
//             vec![Cell { value: 4 }, Cell { value: 5 }, Cell { value: 6 }],
//             vec![Cell { value: 7 }, Cell { value: 8 }, Cell { value: 9 }],
//         ],
//     };

//     // Define test inputs
//     let inputs = vec![
//         "A1=5",                // Assign constant to a cell
//         "A1=-5+2",              // Arithmetic expression
//         "A1=+5-2",              // Arithmetic expression
//         "A1=5*2",              // Arithmetic expression
//         "A1=5/2",              // Arithmetic expression
//         "B2=-A1+A2",               // Assign cell reference
//         "C3=A1+B2",            // Arithmetic expression with cell references
//         "C3=MIN(A1:B2)",       // Function with range
//         "C3=MIN(A1::B2)",       // Function with range
//         "C3=MIN(A1:b2)",       // Function with range
//         "C3=MIN(A1:B2))",       // Function with range
//         "C3=MIN(A1:B2",       // Function with range
//         "C3=STDE(A1:B2)",       // Function with range
//         "B2=1+A1A",               // Assign cell reference
//         "B2=1+A1A1",               // Assign cell reference
//         "B2=1+",               // Assign cell reference // SHOWING IT CORRECT
//         "B2=1+A1+",               // Assign cell reference
//         "B2=1+*A1",               // Assign cell reference
//         "B2=SUM(A1)",               // Assign cell reference
//         "B2=A1:B2+1",               // Assign cell reference
//         "B2=A1(A2)",               // Assign cell reference
//         "B2=B1+A",               // Assign cell reference
//         "C2=SLEEP(10)",        // Function with constant
//         "F6=INVALID_FUNCTION", // Invalid function
//     ];

//     // Iterate over inputs and parse them
//     for input in inputs {
//         println!("Processing input: {}", input);

//         // Reset global variables
//         unsafe {
//             dependency_graph_final::STATUS = 0;
//             *operation_id = 0;
//             *count_operands = 0;
//             *formula = None;
//         }

//         // Parse the input
//         parse_input(input, &mut spreadsheet, 3, 3);

//         // Print the results
//         unsafe {
//             println!("dependency_graph_final::STATUS: {}", dependency_graph_final::STATUS);
//             println!("*operation_id: {}", *operation_id);
//             println!("*count_operands: {}", *count_operands);

//             if let Some(*formula) = &*formula {
//                 println!("*formula:");
//                 for operand in *formula {
//                     match &operand.operand_value {
//                         OperandValue::Constant(value) => {
//                             println!("  Type: Constant, Value: {}", value);
//                         }
//                         OperandValue::CellOperand(cell) => {
//                             println!("  Type: CellOperand, Value: {}", cell.value);
//                         }
//                     }
//                 }
//             } else {
//                 println!("*formula: None");
//             }
//         }

//         println!("-----------------------------------");
//     }
// }
