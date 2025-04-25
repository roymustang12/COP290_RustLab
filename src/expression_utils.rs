use crate::cell_extension::SpreadsheetExtension;
use crate::cellsp::CellReference;
use crate::expression_parser::Expr;
use crate::formula::FormulaParser;
use crate::graph_extension::STATUS_EXTENSION;
use std::str;

/// Parses a formula string into an expression.
///
/// # Arguments
/// * `input` - A string slice containing the formula to parse.
///
/// # Returns
/// * `Ok(Box<Expr>)` - The parsed expression if successful.
/// * `Err(String)` - An error message if parsing fails.
pub fn parse_formula(input: &str) -> Result<Box<Expr>, String> {
    let parser = FormulaParser::new();
    parser
        .parse(input)
        .map_err(|e| format!("Parse error: {:?}", e))
}

/// Recursively extracts precedents (cell references) from an expression.
///
/// # Arguments
/// * `expr` - A reference to the expression to analyze.
/// * `acc` - A mutable vector to accumulate the extracted cell references.
///
/// # Behavior
/// This function traverses the expression tree and collects all cell references
/// (e.g., `A1`, `B2`) and ranges (e.g., `A1:B2`) into the accumulator.
pub fn extract_precedents_helper(expr: &Expr, acc: &mut Vec<CellReference>) {
    match expr {
        Expr::Number(_) => {}

        Expr::Cell(cell_ref) => {
            acc.push(cell_ref.clone());
        }

        Expr::BinaryOp(left, _, right) => {
            extract_precedents_helper(left, acc);
            extract_precedents_helper(right, acc);
        }

        Expr::Function(_, precedents) => {
            for prec in precedents {
                extract_precedents_helper(prec, acc);
            }
        }

        Expr::Range(start, end) => {
            for r in start.row..=end.row {
                for c in start.column..=end.column {
                    acc.push(CellReference { row: r, column: c });
                }
            }
        }
    }
}

/// Extracts all precedents (cell references) from an expression.
///
/// # Arguments
/// * `expr` - A reference to the expression to analyze.
///
/// # Returns
/// A vector of `CellReference` objects representing all precedents in the expression.
pub fn extract_precedents(expr: &crate::expression_parser::Expr) -> Vec<CellReference> {
    let mut acc = Vec::new();
    extract_precedents_helper(expr, &mut acc);
    acc
}

/// Evaluates an expression in the context of a spreadsheet.
///
/// # Arguments
/// * `expr` - A reference to the expression to evaluate.
/// * `sheet` - A reference to the spreadsheet containing cell values.
///
/// # Returns
/// The evaluated result as an integer.
///
/// # Behavior
/// * Supports basic arithmetic operations (`+`, `-`, `*`, `/`).
/// * Supports functions like `SUM`, `MAX`, `MIN`, `AVG`, and `STDEV`.
/// * Handles ranges (e.g., `A1:B2`) for functions like `SUM`.
/// * Returns `0` for invalid operations or division by zero.
pub fn eval_expr(expr: &Expr, sheet: &SpreadsheetExtension) -> i32 {
    match expr {
        Expr::Number(value) => *value,

        Expr::Cell(cell_ref) => {
            sheet.all_cells[cell_ref.row as usize][cell_ref.column as usize].value
        }

        Expr::BinaryOp(left, op, right) => {
            match op {
                '+' => eval_expr(left, sheet) + eval_expr(right, sheet),
                '-' => eval_expr(left, sheet) - eval_expr(right, sheet),
                '*' => eval_expr(left, sheet) * eval_expr(right, sheet),
                '/' => {
                    let divisor = eval_expr(right, sheet);
                    if divisor == 0 {
                        unsafe {
                            STATUS_EXTENSION = 2; // Error status for division by zero
                        }
                        0 // Return 0 for division by zero
                    } else {
                        eval_expr(left, sheet) / divisor
                    }
                }
                _ => unsafe {
                    STATUS_EXTENSION = 12;
                    panic!("Not a valid binary operation");
                },
            }
        }

        Expr::Function(name, args) => {
            match name.as_str() {
                "SUM" | "Sum" => {
                    let mut sum = 0;
                    for arg in args {
                        match arg {
                            Expr::Range(start, end) => {
                                for r in start.row..=end.row {
                                    for c in start.column..=end.column {
                                        // let cell_ref = CellReference { row: r, column: c };
                                        sum += sheet.all_cells[r as usize][c as usize].value;
                                    }
                                }
                            }
                            _ => sum += eval_expr(arg, sheet),
                        }
                    }
                    sum
                }

                "MAX" | "Max" => {
                    let mut values = Vec::new();

                    for arg in args {
                        match arg {
                            Expr::Range(start, end) => {
                                for r in start.row..=end.row {
                                    for c in start.column..=end.column {
                                        values.push(sheet.all_cells[r as usize][c as usize].value);
                                    }
                                }
                            }
                            _ => values.push(eval_expr(arg, sheet)),
                        }
                    }

                    if values.is_empty() {
                        0
                    } else {
                        *values.iter().max().unwrap_or(&0)
                    }
                }

                "MIN" | "Min" => {
                    let mut values = Vec::new();

                    for arg in args {
                        match arg {
                            Expr::Range(start, end) => {
                                for r in start.row..=end.row {
                                    for c in start.column..=end.column {
                                        values.push(sheet.all_cells[r as usize][c as usize].value);
                                    }
                                }
                            }
                            _ => values.push(eval_expr(arg, sheet)),
                        }
                    }

                    if values.is_empty() {
                        0
                    } else {
                        *values.iter().min().unwrap_or(&0)
                    }
                }

                "AVG" | "Avg" => {
                    let mut values = Vec::new();

                    for arg in args {
                        match arg {
                            Expr::Range(start, end) => {
                                for r in start.row..=end.row {
                                    for c in start.column..=end.column {
                                        values.push(sheet.all_cells[r as usize][c as usize].value);
                                    }
                                }
                            }
                            _ => values.push(eval_expr(arg, sheet)),
                        }
                    }

                    if values.is_empty() {
                        0
                    } else {
                        values.iter().sum::<i32>() / values.len() as i32
                    }
                }

                "STDEV" | "Stdev" => {
                    let mut values = Vec::new();

                    for arg in args {
                        match arg {
                            Expr::Range(start, end) => {
                                for r in start.row..=end.row {
                                    for c in start.column..=end.column {
                                        values.push(sheet.all_cells[r as usize][c as usize].value);
                                    }
                                }
                            }
                            _ => values.push(eval_expr(arg, sheet)),
                        }
                    }

                    if values.is_empty() {
                        0
                    } else {
                        // Calculate mean
                        let n = values.len() as f64;
                        let mean: f64 = values.iter().map(|&x| x as f64).sum::<f64>() / n;

                        // Calculate variance
                        let variance = values
                            .iter()
                            .map(|&x| {
                                let diff = (x as f64) - mean;
                                diff * diff
                            })
                            .sum::<f64>()
                            / n;

                        // Return standard deviation as integer
                        variance.sqrt() as i32
                    }
                }

                "SLEEP" | "Sleep" => {
                    if let Some(arg) = args.first() {
                        eval_expr(arg, sheet)
                    } else {
                        0
                    }
                }

                _ => {
                    unsafe {
                        STATUS_EXTENSION = 1;
                    }
                    0
                }
            }
        }

        Expr::Range(_, _) => {
            unsafe {
                STATUS_EXTENSION = 1;
            }
            0
        }
    }
}
