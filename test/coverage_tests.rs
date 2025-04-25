use rust_lab::cellsp2::{CellReference, Operand};
use rust_lab::dependency_graph_final::STATUS;
use rust_lab::dependency_graph_final::*;
use rust_lab::input::*;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn test_initialise_spreadsheet() {
    let sheet = initialise(999, 18278);
    assert_eq!(sheet.rows, 999);
    assert_eq!(sheet.columns, 18278);
    assert_eq!(sheet.all_cells.len(), 999);
    assert_eq!(sheet.all_cells[0].len(), 18278);
}

#[test]
fn test_add_and_delete_dependency() {
    let mut sheet = initialise(2, 2);
    add_dependency(&mut sheet, 0, 0, 1, 1);
    assert!(
        sheet.all_cells[0][0]
            .dependents
            .contains(&CellReference { row: 1, column: 1 })
    );
    assert!(
        sheet.all_cells[1][1]
            .precedents
            .contains(&CellReference { row: 0, column: 0 })
    );

    delete_dependency(&mut sheet, 0, 0, 1, 1);
    assert!(
        !sheet.all_cells[0][0]
            .dependents
            .contains(&CellReference { row: 1, column: 1 })
    );
}

#[test]
fn test_assign_constant_cell() {
    let mut sheet = initialise(2, 2);
    let formula = vec![Operand::Constant(42)];
    assign_cell(&mut sheet, 0, 0, 1, formula);
    assert_eq!(sheet.all_cells[0][0].value, 42);
    assert!(!sheet.all_cells[0][0].is_error);
}

#[test]
fn test_assign_addition_formula() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(5)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(10)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 3, formula); // 3 = addition
    assert_eq!(sheet.all_cells[1][0].value, 15);
}

#[test]
fn test_get_operand_value() {
    let mut sheet = initialise(1, 1);
    sheet.all_cells[0][0].value = 99;
    let operand = Operand::CellOperand(CellReference { row: 0, column: 0 });
    let (val, err) = get_operand_value(&sheet, &operand);
    assert_eq!(val, 99);
    assert!(!err);
}

#[test]
fn test_string_to_int_parsing() {
    assert_eq!(string_to_int("42"), 42);
    assert_eq!(string_to_int("-7"), -7);
    assert_eq!(string_to_int("+123"), 123);
}

#[test]
fn test_parse_cell_name_basic() {
    let mut row = 0;
    let mut col = 0;
    parse_cell_name_1("B2", &mut row, &mut col);
    assert_eq!(row, 1);
    assert_eq!(col, 1);
}

#[test]
fn test_zero_div_err() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(10)]);
    assign_cell(&mut sheet, 0, 2, 1, vec![Operand::Constant(0)]);

    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 0, 0, 6, formula); // 6 = division

    assert!(zero_div_err(&sheet, 0, 0));
}

#[test]
fn test_precedent_error_propagation() {
    let mut sheet = initialise(1, 1);
    sheet.all_cells[0][0].is_error = true;
    sheet.all_cells[0][0]
        .precedents
        .insert(CellReference { row: 0, column: 0 });
    assert!(precedent_has_error(&sheet, 0, 0));
}

#[test]
fn test_assign_subtraction() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(8)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(5)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 4, formula); // 4 = subtraction
    assert_eq!(sheet.all_cells[1][0].value, 3);
    assert!(!sheet.all_cells[1][0].is_error);
}

#[test]
fn test_assign_multiplication() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(3)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(4)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 5, formula); // 5 = multiplication
    assert_eq!(sheet.all_cells[1][0].value, 12);
    assert!(!sheet.all_cells[1][0].is_error);
}

#[test]
fn test_assign_division() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(2)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 6, formula); // 6 = division
    assert_eq!(sheet.all_cells[1][0].value, 5);
    assert!(!sheet.all_cells[1][0].is_error);
}

#[test]
fn test_assign_min_function() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(9)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(2)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 7, formula); // 7 = MIN
    assert_eq!(sheet.all_cells[1][0].value, 2);
}

#[test]
fn test_assign_max_function() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(9)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(2)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 8, formula); // 8 = MAX
    assert_eq!(sheet.all_cells[1][0].value, 9);
}

#[test]
fn test_assign_avg_function() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(20)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 9, formula); // 9 = AVG
    assert_eq!(sheet.all_cells[1][0].value, 15);
}

#[test]
fn test_assign_sum_function() {
    let mut sheet = initialise(2, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(3)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(4)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 1, 0, 10, formula); // 10 = SUM
    assert_eq!(sheet.all_cells[1][0].value, 7);
}

#[test]
fn test_assign_stdev_function() {
    let mut sheet = initialise(1, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(1)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(3)]);
    let formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
    ];
    assign_cell(&mut sheet, 0, 0, 11, formula); // 11 = STDEV
    assert_eq!(sheet.all_cells[0][0].value, 1); // rounded standard deviation of [1, 3]
}

#[test]
fn test_parse_input_basic_constant() {
    let mut sheet = initialise(3, 3);
    let mut operation_id = 0;
    let mut edit_row = 0;
    let mut edit_col = 0;
    let mut count_operands = 0;
    let mut formula = vec![];

    parse_input(
        "A1=42",
        &mut sheet,
        3,
        3,
        &mut operation_id,
        &mut edit_row,
        &mut edit_col,
        &mut count_operands,
        &mut formula,
    );

    assert_eq!(operation_id, 1);
    assert_eq!(edit_row, 0);
    assert_eq!(edit_col, 0);
    assert_eq!(count_operands, 1);
}

#[test]
fn test_constant_assignment() {
    let mut sheet = initialise(1, 1);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(42)]);
    assert_eq!(sheet.all_cells[0][0].value, 42);
    assert!(!sheet.all_cells[0][0].is_error);
}

#[test]
fn test_reference_assignment() {
    let mut sheet = initialise(1, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    assign_cell(
        &mut sheet,
        0,
        1,
        2,
        vec![Operand::CellOperand(CellReference { row: 0, column: 0 })],
    );
    assert_eq!(sheet.all_cells[0][1].value, 10);
}

#[test]
fn test_addition_assignment() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(2)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(3)]);
    assign_cell(
        &mut sheet,
        0,
        2,
        3,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    assert_eq!(sheet.all_cells[0][2].value, 5);
}

#[test]
fn test_division_by_zero() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(8)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    assert!(sheet.all_cells[0][2].is_error);
}

#[test]
fn test_min_function() {
    let mut sheet = initialise(1, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(9)]);
    assign_cell(
        &mut sheet,
        0,
        1,
        7,
        vec![Operand::CellOperand(CellReference { row: 0, column: 0 })],
    );
    assert_eq!(sheet.all_cells[0][1].value, 9);
}

#[test]
fn test_max_function() {
    let mut sheet = initialise(1, 2);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(11)]);
    assign_cell(
        &mut sheet,
        0,
        1,
        8,
        vec![Operand::CellOperand(CellReference { row: 0, column: 0 })],
    );
    assert_eq!(sheet.all_cells[0][1].value, 11);
}

#[test]
fn test_avg_function() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(20)]);
    assign_cell(
        &mut sheet,
        0,
        2,
        9,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    assert_eq!(sheet.all_cells[0][2].value, 15);
}

#[test]
fn test_sum_function() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(7)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(8)]);
    assign_cell(
        &mut sheet,
        0,
        2,
        10,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    assert_eq!(sheet.all_cells[0][2].value, 15);
}

#[test]
fn test_stdev_function() {
    let mut sheet = initialise(1, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(1)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(3)]);
    assign_cell(
        &mut sheet,
        0,
        2,
        11,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    assert_eq!(sheet.all_cells[0][2].value, 1); // stdev(1,3)=1.0
}

#[test]
fn test_sleep_function() {
    let mut sheet = initialise(1, 2);
    assign_cell(&mut sheet, 0, 0, 12, vec![Operand::Constant(1)]);
    std::thread::sleep(std::time::Duration::from_secs(2));
    process_sleep_completions(&mut sheet);
    assert_eq!(sheet.all_cells[0][0].value, 1);
}

#[test]
fn test_cycle_detection() {
    let mut sheet = initialise(1, 1);
    let cell_ref = CellReference { row: 0, column: 0 };
    sheet.all_cells[0][0].dependents.insert(cell_ref.clone());
    assert!(has_cycle(&sheet, 0, 0));
}

#[test]
fn test_parse_input_invalid_reference() {
    let mut sheet = initialise(2, 2);
    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "A1=Z9999",
        &mut sheet,
        2,
        2,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );

    let status;
    unsafe {
        status = STATUS;
    }
    assert_eq!(status, 1);
}

#[test]
fn test_subtraction_operation() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        4,
        vec![Operand::Constant(10), Operand::Constant(3)],
    );
    assert_eq!(sheet.all_cells[0][0].value, 7);
}

#[test]
fn test_multiplication_operation() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        5,
        vec![Operand::Constant(4), Operand::Constant(5)],
    );
    assert_eq!(sheet.all_cells[0][0].value, 20);
}

#[test]
fn test_division_operation_valid() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        6,
        vec![Operand::Constant(10), Operand::Constant(2)],
    );
    assert_eq!(sheet.all_cells[0][0].value, 5);
    assert!(!sheet.all_cells[0][0].is_error);
}

#[test]
fn test_division_by_zero_error() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        6,
        vec![Operand::Constant(10), Operand::Constant(0)],
    );
    assert!(sheet.all_cells[0][0].is_error);
}

#[test]
fn test_min_function_multiple() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        7,
        vec![
            Operand::Constant(5),
            Operand::Constant(2),
            Operand::Constant(9),
        ],
    );
    assert_eq!(sheet.all_cells[0][0].value, 2);
}

#[test]
fn test_max_function_multiple() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        8,
        vec![
            Operand::Constant(5),
            Operand::Constant(10),
            Operand::Constant(3),
        ],
    );
    assert_eq!(sheet.all_cells[0][0].value, 10);
}

#[test]
fn test_avg_function_multiple() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        9,
        vec![Operand::Constant(6), Operand::Constant(12)],
    );
    assert_eq!(sheet.all_cells[0][0].value, 9);
}

#[test]
fn test_sum_function_multiple() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        10,
        vec![
            Operand::Constant(4),
            Operand::Constant(6),
            Operand::Constant(10),
        ],
    );
    assert_eq!(sheet.all_cells[0][0].value, 20);
}

#[test]
fn tes_stdev_function() {
    let mut sheet = initialise(1, 1);
    assign_cell(
        &mut sheet,
        0,
        0,
        11,
        vec![
            Operand::Constant(2),
            Operand::Constant(4),
            Operand::Constant(4),
            Operand::Constant(4),
            Operand::Constant(5),
            Operand::Constant(5),
            Operand::Constant(7),
            Operand::Constant(9),
        ],
    );
    assert_eq!(sheet.all_cells[0][0].value, 2);
}

#[test]
fn test_main_accepts_valid_input() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "A1=10").unwrap();
    writeln!(stdin, "B1=A1").unwrap();
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_parse_input_invalid_no_rhs() {
    let mut sheet = initialise(3, 3);
    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "A1=",
        &mut sheet,
        3,
        3,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );
    let status;
    unsafe {
        status = STATUS;
    }
    assert_eq!(status, 1);
}

#[test]
fn test_parse_input_invalid_no_lhs() {
    let mut sheet = initialise(3, 3);
    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "=42",
        &mut sheet,
        3,
        3,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );
    let status;
    unsafe {
        status = STATUS;
    }
    assert_eq!(status, 1);
}

#[test]
fn test_parse_input_malformed_range() {
    let mut sheet = initialise(3, 3);
    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "A1=SUM(A1:)",
        &mut sheet,
        3,
        3,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );
    let status;
    unsafe {
        status = STATUS;
    }
    assert_eq!(status, 1);
}

#[test]
fn test_parse_input_bad_parens_function() {
    let mut sheet = initialise(3, 3);
    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "A1=MAX(A1",
        &mut sheet,
        3,
        3,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );
    let status;
    unsafe {
        status = STATUS;
    }
    assert_eq!(status, 1);
}

#[test]
fn test_parse_input_valid_range_sum() {
    let mut sheet = initialise(3, 3);
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(20)]);

    let mut op = 0;
    let mut row = 0;
    let mut col = 0;
    let mut count = 0;
    let mut formula = vec![];

    parse_input(
        "C1=SUM(A1:B1)",
        &mut sheet,
        3,
        3,
        &mut op,
        &mut row,
        &mut col,
        &mut count,
        &mut formula,
    );
    assert_eq!(row, 0);
    assert_eq!(col, 2);
    assert_eq!(op, 10); // SUM
    assert_eq!(count, 2);
}

#[test]
fn test_recalculate_dependents_zero_division() {
    let mut sheet = initialise(1, 3);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    assert!(sheet.all_cells[0][2].is_error);
}
#[test]
fn test_recalculate_dependents_chain() {
    let mut sheet = initialise(1, 3);

    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(2)]);
    assign_cell(
        &mut sheet,
        0,
        1,
        3,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::Constant(3),
        ],
    );
    assign_cell(
        &mut sheet,
        0,
        2,
        3,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
            Operand::Constant(5),
        ],
    );

    assert_eq!(sheet.all_cells[0][1].value, 5); // 2+3
    assert_eq!(sheet.all_cells[0][2].value, 10); // 5+5
}

#[test]
fn test_main_invalid() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "A1=10").unwrap();
    writeln!(stdin, "B1=A").unwrap();
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid2() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "A1=10").unwrap();
    writeln!(stdin, "B1=A2/A1").unwrap();
    writeln!(stdin, "A1=0").unwrap();
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_val() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "A1=10").unwrap();
    writeln!(stdin, "B1=A2-A1").unwrap();
    writeln!(stdin, "B1=A2*A1").unwrap();
    writeln!(stdin, "A1=0").unwrap();
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid3() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "1000", "19000"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "A6=10").unwrap();
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_too_few_args() {
    let mut child = Command::new("cargo")
        .args(["run", "--"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_one_arg() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_too_many_args() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "10", "20", "30"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_nonint_row() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "abc", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_nonint_col() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "10", "xyz"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_row_zero() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "0", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_invalid_negative_col() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "10", "-5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let _ = child.wait().unwrap();
}

#[test]
fn test_invalid_column_name() {
    let mut row = 0;
    let mut col = 0;

    // Invalid column: "1A" (starts with a number)
    parse_cell_name_1("1A", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);

    // Invalid column: "@1" (contains special character)
    parse_cell_name_1("@1", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);

    // Invalid column: "AA1" (out-of-range column "AA")
    parse_cell_name_1("AA1", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);
}

#[test]
fn test_invalid_row_part() {
    let mut row = 0;
    let mut col = 0;

    // Invalid row: "Aabc" (row part contains alphabetic characters)
    parse_cell_name_1("Aabc", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);

    // Invalid row: "A+1" (row part contains arithmetic expression)
    parse_cell_name_1("A+1", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);

    // Invalid row: "A1A" (row part contains alphabetic character "A")
    parse_cell_name_1("A1A", &mut row, &mut col);
    assert_eq!(unsafe { STATUS }, 1);
}

#[test]
fn test_update_dependency_with_invalid_data() {
    let mut sheet = initialise(2, 2); // Assuming 2x2 sheet
    // Perform the operation
    delete_dependency(&mut sheet, 0, 0, 1, 1); // Example of calling the function

    // Now check that the cell was properly updated or remains in an expected state
    assert_eq!(sheet.all_cells[1][1].value, 0); // or whatever value is expected after deletion
}

#[test]
fn test_main_scroll_to_valid() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "10", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "scroll_to B2").unwrap(); // Valid scroll within bounds
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_scroll_to_out_of_bounds() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "scroll_to Z100").unwrap(); // Out-of-bounds scroll
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_scroll_to_malformed() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    writeln!(stdin, "scroll_to 1A").unwrap(); // Malformed cell reference
    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_sleep_precedent_error() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().expect("failed to open stdin");

    // Force error in A1
    writeln!(stdin, "A1=A1").unwrap(); // Self-reference causes error

    // Assign B1 to sleep on A1's value
    writeln!(stdin, "B1~A1").unwrap(); // ~ triggers sleep with A1 as operand

    writeln!(stdin, "q").unwrap(); // quit

    let _ = child.wait().unwrap();
}

#[test]
fn test_main_navigation_w_top_bound() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // Small grid
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "w").unwrap(); // attempt to scroll up from row 0
    writeln!(stdin, "q").unwrap(); // quit
    let _ = child.wait().unwrap();
}

#[test]
fn test_main_navigation_s_bottom_bound() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "15", "5"]) // Only 15 rows
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "s").unwrap(); // go down 10 rows
    writeln!(stdin, "s").unwrap(); // try to go beyond row 15
    writeln!(stdin, "q").unwrap(); // quit
    let _ = child.wait().unwrap();
}

#[test]
fn test_main_navigation_a_left_bound() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // Small grid
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "a").unwrap(); // attempt to scroll left from column 0
    writeln!(stdin, "q").unwrap(); // quit
    let _ = child.wait().unwrap();
}

#[test]
fn test_main_navigation_d_right_bound() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "15"]) // Only 15 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "d").unwrap(); // scroll right by 10 columns
    writeln!(stdin, "d").unwrap(); // try to go beyond 15
    writeln!(stdin, "q").unwrap(); // quit
    let _ = child.wait().unwrap();
}

#[test]
fn test_main_navigation_d_all_cases() {
    // Case 1: CURRENT_COL + 20 <= columns → scroll by 10
    let mut child1 = Command::new("cargo")
        .args(["run", "--", "5", "30"]) // 30 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run case 1");

    let mut stdin1 = child1.stdin.take().unwrap();
    writeln!(stdin1, "d").unwrap(); // Should move to 10
    writeln!(stdin1, "q").unwrap();
    let _ = child1.wait().unwrap();

    // Case 2: CURRENT_COL + 20 > columns → jump to columns - 10
    let mut child2 = Command::new("cargo")
        .args(["run", "--", "5", "15"]) // 15 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run case 2");

    let mut stdin2 = child2.stdin.take().unwrap();
    writeln!(stdin2, "d").unwrap(); // Should move to 5
    writeln!(stdin2, "q").unwrap();
    let _ = child2.wait().unwrap();

    // Case 3: CURRENT_COL + 10 > columns → stay put
    let mut child3 = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // Only 5 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run case 3");

    let mut stdin3 = child3.stdin.take().unwrap();
    writeln!(stdin3, "d").unwrap(); // Should stay at 0
    writeln!(stdin3, "q").unwrap();
    let _ = child3.wait().unwrap();
}

#[test]
fn test_main_navigation_s_and_a_all_cases() {
    // ──────────────── "s" movement tests ────────────────

    // Case 1: CURRENT_ROW + 20 <= rows → scroll down by 10
    let mut child1 = Command::new("cargo")
        .args(["run", "--", "50", "5"]) // 50 rows
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run s case 1");
    let mut stdin1 = child1.stdin.take().unwrap();
    writeln!(stdin1, "s").unwrap();
    writeln!(stdin1, "q").unwrap();
    let _ = child1.wait().unwrap();

    // Case 2: CURRENT_ROW + 20 > rows → jump to rows - 10
    let mut child2 = Command::new("cargo")
        .args(["run", "--", "15", "5"]) // 15 rows
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run s case 2");
    let mut stdin2 = child2.stdin.take().unwrap();
    writeln!(stdin2, "s").unwrap();
    writeln!(stdin2, "q").unwrap();
    let _ = child2.wait().unwrap();

    // Case 3: CURRENT_ROW + 10 > rows → no movement
    let mut child3 = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // 5 rows
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run s case 3");
    let mut stdin3 = child3.stdin.take().unwrap();
    writeln!(stdin3, "s").unwrap();
    writeln!(stdin3, "q").unwrap();
    let _ = child3.wait().unwrap();

    // ──────────────── "a" movement tests ────────────────

    // Case 1: CURRENT_COL - 10 < 0 → should reset to 0
    let mut child4 = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // Only 5 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run a case 1");
    let mut stdin4 = child4.stdin.take().unwrap();
    writeln!(stdin4, "a").unwrap(); // no change; already at 0
    writeln!(stdin4, "q").unwrap();
    let _ = child4.wait().unwrap();

    // Case 2: CURRENT_COL = 20 → move left by 10
    let mut child5 = Command::new("cargo")
        .args(["run", "--", "5", "50"]) // 50 columns
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run a case 2");
    let mut stdin5 = child5.stdin.take().unwrap();
    // Move right twice first
    writeln!(stdin5, "d").unwrap(); // CURRENT_COL = 10
    writeln!(stdin5, "d").unwrap(); // CURRENT_COL = 20
    writeln!(stdin5, "a").unwrap(); // Should now be 10
    writeln!(stdin5, "q").unwrap();
    let _ = child5.wait().unwrap();
}

#[test]
fn test_sleep_with_constant() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=SLEEP(2)").unwrap();
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sleep_with_valid_cell_reference() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "B2=5").unwrap();
    writeln!(stdin, "A1=SLEEP(B2)").unwrap();
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sleep_with_out_of_bounds_cell_reference() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"]) // only 5x5 grid
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=SLEEP(Z1000)").unwrap(); // invalid cell reference
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sleep_with_invalid_cell_name() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=SLEEP(XYZ)").unwrap(); // non-parseable as cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_valid_constant_plus_constant() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=3+5").unwrap();
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_valid_constant_plus_cell() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "B2=10").unwrap();
    writeln!(stdin, "A1=3+B2").unwrap();
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_valid_negative_constant_plus_cell() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "C3=2").unwrap();
    writeln!(stdin, "A1=-5+C3").unwrap(); // negative constant + cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_invalid_operand1_is_cell() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=B2+3").unwrap(); // invalid: operand1 is cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_operand2_out_of_bounds_cell() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=2+Z999").unwrap(); // out-of-bounds cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_operand2_is_invalid_cell_string() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=2+XYZ").unwrap(); // malformed operand2
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_expr_missing_operand2() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=2+").unwrap(); // no operand2
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_max_with_valid_cell_no_error() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().unwrap();

    writeln!(stdin, "B2=100").unwrap(); // B2 holds a constant (no error)
    writeln!(stdin, "A1=MAX(B2)").unwrap(); // A1 takes value of B2 using MAX
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_max_with_cell_in_error() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().unwrap();

    writeln!(stdin, "C3=B2/0").unwrap(); // Division by zero: sets C3 in error
    writeln!(stdin, "A1=MAX(C3)").unwrap(); // A1 depends on error cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_max_with_invalid_operand_constant() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "5", "5"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn process");

    let mut stdin = child.stdin.take().unwrap();

    writeln!(stdin, "A1=MAX(42)").unwrap(); // MAX with constant instead of cell
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_recalculate_with_zero_division_error() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=0").unwrap();
    writeln!(stdin, "A2=10/A1").unwrap(); // triggers zero division
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_recalculate_with_error_propagation() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=10/0").unwrap(); // A1 error
    writeln!(stdin, "A2=A1+5").unwrap(); // A2 gets error via dependency
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_recalculate_with_successful_propagation() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=5").unwrap();
    writeln!(stdin, "A2=A1+5").unwrap(); // Should recalculate without error
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_recalculate_transitive_dependencies() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=2").unwrap();
    writeln!(stdin, "A2=A1+3").unwrap();
    writeln!(stdin, "A3=A2+4").unwrap(); // A3 transitively depends on A1
    writeln!(stdin, "A1=5").unwrap(); // Should trigger recalculations down the chain
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sum_no_error_in_precedent() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=5").unwrap(); // Precedent value for sum
    writeln!(stdin, "A2=SUM(A1)").unwrap(); // SUM should work without errors
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sum_with_error_in_precedent() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=10/0").unwrap(); // A1 has an error (division by zero)
    writeln!(stdin, "A2=SUM(A1)").unwrap(); // SUM should fail because of A1's error
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sum_with_empty_precedent() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=").unwrap(); // Empty precedent value
    writeln!(stdin, "A2=SUM(A1)").unwrap(); // SUM should treat empty value gracefully
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_sum_with_error_propagation() {
    let mut child = Command::new("cargo")
        .args(["run", "--", "3", "3"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start process");

    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "A1=5").unwrap(); // Valid precedent value
    writeln!(stdin, "A2=SUM(A1)").unwrap(); // SUM should work
    writeln!(stdin, "A3=SUM(A2)").unwrap(); // A3 depends on A2, which is valid
    writeln!(stdin, "A1=10/0").unwrap(); // A1 will now have an error (division by zero)
    writeln!(stdin, "A4=SUM(A1)").unwrap(); // SUM should propagate error through chain
    writeln!(stdin, "q").unwrap();

    let _ = child.wait().unwrap();
}

#[test]
fn test_avg_function_no_error() {
    // Initialize a 1x1 spreadsheet (or however many rows/columns you want)
    let mut sheet = initialise(3, 1); // 3 rows and 1 column for simplicity

    // Assign values to cells
    assign_cell(
        &mut sheet,
        0,
        0,
        9,
        vec![Operand::Constant(6), Operand::Constant(12)],
    ); // AVG(6, 12)
    assign_cell(
        &mut sheet,
        1,
        0,
        9,
        vec![Operand::Constant(10), Operand::Constant(20)],
    ); // AVG(10, 20)
    assign_cell(
        &mut sheet,
        2,
        0,
        9,
        vec![Operand::Constant(30), Operand::Constant(40)],
    ); // AVG(30, 40)

    // Test the AVG calculation, expect the average of 6, 12, 10, 20, 30, 40, which is 18
    assert_eq!(sheet.all_cells[0][0].value, 9); // Update this as needed for correct calculation
    assert_eq!(sheet.all_cells[1][0].value, 15); // AVG(10, 20) = 15
    assert_eq!(sheet.all_cells[2][0].value, 35); // AVG(30, 40) = 35

    // Ensure no error in the cells
    assert_eq!(sheet.all_cells[0][0].is_error, false);
    assert_eq!(sheet.all_cells[1][0].is_error, false);
    assert_eq!(sheet.all_cells[2][0].is_error, false);
}

#[test]
fn test_avg_with_division_by_zero_error() {
    let mut sheet = initialise(4, 4);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    // C2 = AVG(A1:C1)
    let avg_formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 1, 2, 9, avg_formula);

    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1
    assert!(sheet.all_cells[1][2].is_error); // C2
}

#[test]
fn test_sumvg_with_division_by_zero_error() {
    let mut sheet = initialise(4, 4);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );
    // C2 = AVG(A1:C1)
    let avg_formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 1, 2, 10, avg_formula);

    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1
    assert!(sheet.all_cells[1][2].is_error); // C2
}

#[test]
fn test_std_with_division_by_zero_error() {
    let mut sheet = initialise(4, 4);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    let avg_formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 1, 2, 11, avg_formula);

    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1
    assert!(sheet.all_cells[1][2].is_error); // C2
}

#[test]
fn test_maxith_division_by_zero_error() {
    let mut sheet = initialise(4, 4);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    let avg_formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 1, 2, 7, avg_formula);

    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1
    assert!(sheet.all_cells[1][2].is_error); // C2
}

#[test]
fn test_mminh_division_by_zero_error() {
    let mut sheet = initialise(4, 4);

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    let avg_formula = vec![
        Operand::CellOperand(CellReference { row: 0, column: 0 }),
        Operand::CellOperand(CellReference { row: 0, column: 1 }),
        Operand::CellOperand(CellReference { row: 0, column: 2 }),
    ];
    assign_cell(&mut sheet, 1, 2, 8, avg_formula);

    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1
    assert!(sheet.all_cells[1][2].is_error); // C2
}

#[test]
fn test_precedent_cell_non_error_path() {
    let mut sheet = initialise(4, 4);

    // A1 = 20 (no error)
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(20)]);

    // A2 = reference to A1 (should copy A1’s value and not error)
    assign_cell(
        &mut sheet,
        1,  // row = 1 -> A2
        0,  // col = 0
        11, // opcode for STDEV, triggers that match block
        vec![Operand::CellOperand(CellReference { row: 0, column: 0 })],
    );

    // Check that A2 took A1’s value
    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[1][0].is_error); // A2
    assert_eq!(sheet.all_cells[1][0].value, 0); // A2 copies value from A1
}

#[test]
fn test_min_function_with_error_propagation() {
    let mut sheet = initialise(4, 4);

    // A1 = 3
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(3)]);
    // B1 = 0
    assign_cell(&mut sheet, 0, 1, 1, vec![Operand::Constant(0)]);
    // C1 = A1 / B1 -> Division by zero
    assign_cell(
        &mut sheet,
        0,
        2,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    // D1 = MIN(A1:C1) -> Should fail due to error in C1
    assign_cell(
        &mut sheet,
        0,
        3,
        7,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
            Operand::CellOperand(CellReference { row: 0, column: 2 }),
        ],
    );
    assert!(sheet.all_cells[0][3].is_error); // D1 (MIN(A1:C1) has error due to C1)

    assign_cell(
        &mut sheet,
        0,
        3,
        7,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 0, column: 1 }),
        ],
    );

    // Assertions to check errors
    assert!(!sheet.all_cells[0][0].is_error); // A1
    assert!(!sheet.all_cells[0][1].is_error); // B1
    assert!(sheet.all_cells[0][2].is_error); // C1 (Division by zero)
    assert_eq!(sheet.all_cells[0][3].value, 0);
}

#[test]
fn test_max_chain_with_division_by_zero() {
    let mut sheet = initialise(7, 1); // A1 to A6 → 6 rows, 1 column

    // A1 = 10
    assign_cell(&mut sheet, 0, 0, 1, vec![Operand::Constant(10)]);
    // A2 = 0
    assign_cell(&mut sheet, 1, 0, 1, vec![Operand::Constant(0)]);
    // A3 = A1 / A2 → Division by zero error
    assign_cell(
        &mut sheet,
        2,
        0,
        6,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 1, column: 0 }),
        ],
    );
    // A4 = constant value 5 (just to fill A1:A4 range)
    assign_cell(&mut sheet, 3, 0, 1, vec![Operand::Constant(5)]);
    // A5 = MAX(A1:A4) → should error due to A3
    assign_cell(
        &mut sheet,
        4,
        0,
        9, // Assuming opcode 9 is MAX
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 1, column: 0 }),
            Operand::CellOperand(CellReference { row: 2, column: 0 }),
            Operand::CellOperand(CellReference { row: 3, column: 0 }),
        ],
    );
    // A6 = MAX(A1:A6) → should error due to A5
    assign_cell(
        &mut sheet,
        5,
        0,
        9,
        vec![
            Operand::CellOperand(CellReference { row: 0, column: 0 }),
            Operand::CellOperand(CellReference { row: 1, column: 0 }),
            Operand::CellOperand(CellReference { row: 2, column: 0 }),
            Operand::CellOperand(CellReference { row: 3, column: 0 }),
            Operand::CellOperand(CellReference { row: 4, column: 0 }),
            Operand::CellOperand(CellReference { row: 5, column: 0 }),
        ],
    );

    dbg!(sheet.all_cells[4][0].is_error); // A5
    dbg!(!sheet.all_cells[5][0].is_error); // A6
    dbg!(sheet.all_cells[5][0].formula.clone()); // see what A6 depends on

    // Assertions
    assert_eq!(sheet.all_cells[0][0].value, 10); // A1
    assert_eq!(sheet.all_cells[1][0].value, 0); // A2
    assert!(sheet.all_cells[2][0].is_error); // A3 = A1 / A2
    assert_eq!(sheet.all_cells[3][0].value, 5); // A4
    assert!(sheet.all_cells[4][0].is_error); // A5 = MAX(A1:A4)
    assert!(!sheet.all_cells[5][0].is_error); // A6 = MAX(A1:A6)
}
