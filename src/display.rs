use fltk::{
    app,
    button::{Button, RadioButton},
    dialog::alert,
    enums::{Align, Color, Font, FrameType},
    frame::Frame,
    input::Input,
    prelude::*,
    window::Window,
};

use crate::cell_extension::SpreadsheetExtension;
use crate::graph_extension::UndoRedoStack;
use crate::parser_visual_mode::parser_visual;
use crate::{expression_parser::Expr, read_mode::handle_read_command};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
pub fn get_column_name(mut col_index: i32) -> String {
    let mut col_name = String::new();
    col_index += 1; // Excel is 1-based

    while col_index > 0 {
        let rem = (col_index - 1) % 26;
        col_name.insert(0, (b'A' + rem as u8) as char);
        col_index = (col_index - 1) / 26;
    }

    col_name
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        // Handle constant integer values
        Expr::Number(value) => value.to_string(),

        // Handle cell references
        Expr::Cell(cell_ref) => format!("{}", cell_ref),

        // Handle binary operations
        Expr::BinaryOp(left, op, right) => {
            format!(
                "({} {} {})",
                expr_to_string(left),
                op,
                expr_to_string(right)
            )
        }

        // Handle functions
        Expr::Function(name, args) => {
            let args_str = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args_str)
        }

        // Handle ranges
        Expr::Range(start, end) => format!("{}:{}", start, end),
    }
}

const CELL_WIDTH: i32 = 100;
const CELL_HEIGHT: i32 = 30;

pub fn launch_gui(
    _data: Arc<Mutex<Vec<Vec<String>>>>,
    sheet: Arc<Mutex<SpreadsheetExtension>>, // Pass the complete sheet
    input_text: Arc<Mutex<String>>,
    status_extension: Arc<Mutex<i32>>,
    current_row: Arc<Mutex<usize>>,
    current_col: Arc<Mutex<usize>>,
    undo_manager: Arc<Mutex<UndoRedoStack>>,
) {
    thread::spawn(move || {
        let app = app::App::default();
        let screen_width = app::screen_size().0 as i32;
        let screen_height = app::screen_size().1 as i32;

        let cols = (screen_width - 20) / CELL_WIDTH;
        let rows = (screen_height - 60) / CELL_HEIGHT - 1;

        let mut wind = Window::new(
            0,
            0,
            screen_width,
            screen_height,
            "Rust Spreadsheet Display",
        );
        wind.fullscreen(true);

        let scroll_btn_size = 30;

        let mut right_btn = Button::new(
            screen_width - 2 * scroll_btn_size - 20,
            screen_height - scroll_btn_size - 30,
            scroll_btn_size,
            scroll_btn_size,
            "→",
        );

        let mut down_btn = Button::new(
            screen_width - 30,
            screen_height - scroll_btn_size - 30,
            scroll_btn_size,
            scroll_btn_size,
            "↓",
        );

        let mut up_btn = Button::new(screen_width - 30, 50, scroll_btn_size, scroll_btn_size, "↑");

        let mut left_btn = Button::new(
            10,
            screen_height - scroll_btn_size - 30,
            scroll_btn_size,
            scroll_btn_size,
            "←",
        );

        // Add Undo and Redo buttons
        let mut undo_btn = Button::new(screen_width - 220, 10, 100, 30, "Undo");
        let mut redo_btn = Button::new(screen_width - 110, 10, 100, 30, "Redo");

        // Undo button callback
        let input_text_clone = input_text.clone();
        undo_btn.set_callback(move |_| {
            let mut input = input_text_clone.lock().unwrap();
            *input = "undo".to_string(); // Send "undo" command to the input handler
            app::awake(); // Trigger the main thread to process the input
        });

        // Redo button callback
        let input_text_clone = input_text.clone();
        redo_btn.set_callback(move |_| {
            let mut input = input_text_clone.lock().unwrap();
            *input = "redo".to_string(); // Send "redo" command to the input handler
            app::awake(); // Trigger the main thread to process the input
        });

        // Add the "Read" button
        // Add the "Read" button
        let mut read_btn = Button::new(screen_width - 440, 10, 100, 30, "Read");

        // Read button callback
        let sheet_clone = sheet.clone();
        let undo_manager_clone = undo_manager.clone();
        // let input_text_clone = input_text.clone();

        read_btn.set_callback(move |_| {
            // Create a popup window for input
            let popup = Rc::new(RefCell::new(Window::new(
                300,
                200,
                400,
                200,
                "Enter Filename",
            )));
            let input = Input::new(50, 50, 300, 30, "Filename:");
            let mut submit_btn = Button::new(150, 120, 100, 30, "Submit");

            // Handle the submit button click
            let sheet_clone_inner = sheet_clone.clone(); // Clone the value for use in the closure
            let undo_manager_clone_inner = undo_manager_clone.clone(); // Clone the undo manager
            let popup_clone = popup.clone(); // Clone Rc for use in the closure
            submit_btn.set_callback(move |_| {
                let filename = input.value();
                if !filename.is_empty() {
                    // Concatenate "read " with the filename
                    let command = format!("read {}", filename);

                    // Send the command to the input_text shared variable
                    let mut sheet = sheet_clone_inner.lock().unwrap();
                    let mut undo_manager = undo_manager_clone_inner.lock().unwrap();
                    handle_read_command(&command, &mut sheet, &mut undo_manager);

                    // Trigger the GUI to refresh
                    app::awake(); // Notify the main thread to update the GUI
                }
                popup_clone.borrow_mut().hide(); // Close the popup after submission
            });
            popup.borrow_mut().end();
            popup.borrow_mut().show();
        });

        // Add the "Plot Graph" button
        let mut plot_btn = Button::new(screen_width - 330, 10, 100, 30, "Plot Graph");

        // Plot Graph button callback
        let sheet_clone = sheet.clone();
        let undo_manager_clone = undo_manager.clone();
        plot_btn.set_callback(move |_| {
            // Create a popup window for input
            let popup = Rc::new(RefCell::new(Window::new(
                300,
                200,
                400,
                300,
                "Enter Graph Input",
            )));
            let input = Input::new(50, 200, 300, 30, "Input:");
            let mut submit_btn = Button::new(150, 260, 100, 30, "Submit");

            // Add radio buttons for graph types
            let mut plot_histogram = RadioButton::new(50, 50, 150, 30, "plot_histogram");
            let mut plot_line = RadioButton::new(50, 80, 150, 30, "plot_line");
            let mut plot_scatter = RadioButton::new(50, 110, 150, 30, "plot_scatter");
            let mut forecast = RadioButton::new(50, 140, 150, 30, "forecast");

            // Group the radio buttons
            plot_histogram.set_value(true); // Default selection
            plot_line.set_value(false);
            plot_scatter.set_value(false);
            forecast.set_value(false);

            // Handle the submit button click
            let sheet_clone = sheet_clone.clone();
            let undo_manager_clone = undo_manager_clone.clone();
            let popup_clone = popup.clone(); // Clone Rc for use in the closure
            submit_btn.set_callback(move |_| {
                let input_value = input.value();
                if !input_value.is_empty() {
                    // Determine which radio button is selected
                    let selected_option = if plot_histogram.value() {
                        "plot_histogram"
                    } else if plot_line.value() {
                        "plot_line"
                    } else if plot_scatter.value() {
                        "plot_scatter"
                    } else if forecast.value() {
                        "forecast"
                    } else {
                        ""
                    };

                    // Concatenate the selected option with the user input
                    let final_input = format!("{} {}", selected_option, input_value);

                    // Call parser_visual with the concatenated input
                    let mut sheet = sheet_clone.lock().unwrap();
                    let mut undo_manager = undo_manager_clone.lock().unwrap();
                    parser_visual(&final_input, &mut sheet, &mut undo_manager);
                }
                popup_clone.borrow_mut().hide(); // Close the popup after submission
            });

            popup.borrow_mut().end();
            popup.borrow_mut().show();
        });

        let mut input = Input::new(10, 10, screen_width - 600, 30, "");
        input.set_align(Align::Left);
        input.set_text_size(14);

        // Create a grid of frames to represent the spreadsheet cells
        let mut frames: Vec<Vec<Frame>> = vec![];
        for row in 0..rows {
            let mut row_frames = vec![];
            for col in 0..cols {
                let cell_x = col * CELL_WIDTH + 10;
                let cell_y = row * CELL_HEIGHT + 50;
                let mut frame = Frame::new(cell_x, cell_y, CELL_WIDTH, CELL_HEIGHT, "");
                frame.set_label_size(12);
                frame.set_align(Align::Inside | Align::Left);
                row_frames.push(frame);
            }
            frames.push(row_frames);
        }

        // Track selected cell
        let selected_cell = Arc::new(Mutex::new((None, None))); // (row, col)
        for row in 0..rows {
            for col in 0..cols {
                let mut frame = frames[row as usize][col as usize].clone();
                let selected_cell = selected_cell.clone();
                let mut input = input.clone();
                let current_row = current_row.clone();
                let current_col = current_col.clone();
                // let data = data.clone(); // Clone the Arc<Mutex<Vec<Vec<String>>>> for use in the closure
                let sheet = sheet.clone(); // Clone the Arc<Mutex<SpreadsheetExtension>> for use in the closure

                frame.handle(move |_, ev| {
                    if ev == fltk::enums::Event::Push {
                        let mut selected = selected_cell.lock().unwrap();
                        *selected = (Some(row), Some(col));

                        // For data cells only (skip headers)
                        if row > 0 && col > 0 {
                            let actual_row = *current_row.lock().unwrap() + row as usize - 1;
                            let actual_col = *current_col.lock().unwrap() + col as usize - 1;

                            // let col_name = get_column_name(actual_col as i32);
                            // let cell_ref = format!("{}{}", col_name, actual_row + 1);

                            // Retrieve the formula from the sheet
                            let sheet_data = sheet.lock().unwrap();
                            if actual_row < sheet_data.rows as usize
                                && actual_col < sheet_data.columns as usize
                            {
                                let cell = &sheet_data.all_cells[actual_row][actual_col];
                                let expr = &cell.formula;
                                let formula = expr_to_string(expr);
                                input.set_value(&format!("={}", formula));
                            } else {
                                input.set_value(""); // Cell not found, clear the input
                            }

                            let _ = input.set_position(input.value().len() as i32);
                        }
                        true
                    } else {
                        false
                    }
                });
            }
        }

        wind.end();
        wind.show();

        // Navigation button callbacks
        {
            let current_row = Arc::clone(&current_row);

            // let app = app::App::default();
            up_btn.set_callback(move |_| {
                // println!("current_row: {:?}", current_row);
                let mut row = current_row.lock().unwrap();
                if *row > 0 {
                    *row -= 1;
                }
                app::awake();
            });
        }

        {
            let current_row = Arc::clone(&current_row);

            down_btn.set_callback(move |_| {
                // println!("current_row: {:?}", current_row);
                let mut row = current_row.lock().unwrap();
                *row += 1;
                app::awake();
            });
        }

        {
            let current_col = Arc::clone(&current_col);

            left_btn.set_callback(move |_| {
                // println!("current_col: {:?}", current_col);
                let mut col = current_col.lock().unwrap();
                if *col > 0 {
                    *col -= 1;
                }
                app::awake();
            });
        }

        {
            let current_col = Arc::clone(&current_col);

            right_btn.set_callback(move |_| {
                // println!("current_col: {:?}", current_col);
                let mut col = current_col.lock().unwrap();
                *col += 1;
                app::awake();
            });
        }

        // Handle input submission
        let input_text = input_text.clone();
        let selected_cell_clone = selected_cell.clone(); // Clone for this closure
        let current_row_clone = current_row.clone(); // Clone for this closure
        let current_col_clone = current_col.clone(); // Clone for this closure

        input.handle(move |i, ev| {
            if ev == fltk::enums::Event::KeyDown
                && fltk::app::event_key() == fltk::enums::Key::Enter
            {
                let mut text = input_text.lock().unwrap();
                let selected = selected_cell_clone.lock().unwrap();
                let current_row_val = *current_row_clone.lock().unwrap();
                let current_col_val = *current_col_clone.lock().unwrap();

                // If a cell is selected and input starts with '=', prepend the cell reference
                if let (Some(sel_row), Some(sel_col)) = *selected {
                    if sel_row > 0 && sel_col > 0 && i.value().starts_with('=') {
                        let actual_row = current_row_val + sel_row as usize - 1;
                        let actual_col = current_col_val + sel_col as usize - 1;
                        let col_name = get_column_name(actual_col as i32);
                        let cell_ref = format!("{}{}", col_name, actual_row + 1);
                        *text = format!("{}={}", cell_ref, &i.value()[1..]);
                    } else {
                        *text = i.value();
                    }
                } else {
                    *text = i.value();
                }

                i.set_value("");
                true
            } else {
                false
            }
        });

        // let mut prev_data : SpreadsheetExtension;
        // Add an idle callback to update the GUI dynamically
        app::add_idle3(move |_| {
            let sheet_data = sheet.lock().unwrap();
            let current_row_val = *current_row.lock().unwrap();
            let current_col_val = *current_col.lock().unwrap();
            let selected_cell = selected_cell.lock().unwrap();

            for (row, frame_row) in frames.iter_mut().enumerate().take(rows as usize) {
                for (col, frame) in frame_row.iter_mut().enumerate().take(cols as usize) {
                    // let frame = &mut frames[row][col];

                    // Reset cell appearance first
                    frame.set_color(Color::White);
                    frame.set_frame(FrameType::EngravedBox);
                    frame.set_label_font(Font::Helvetica);

                    // Handle headers
                    if row == 0 && col == 0 {
                        frame.set_label("@");
                        frame.set_color(Color::Gray0);
                        frame.set_frame(FrameType::FlatBox);
                        continue;
                    }
                    // println!("current_row: {}, current_col: {}", current_row_val, current_col_val);
                    if row == 0 {
                        let col_index = current_col_val + col - 1;
                        let col_name = get_column_name(col_index as i32);
                        frame.set_label(&col_name);
                        frame.set_label_font(Font::HelveticaBold);
                        frame.set_color(Color::from_rgb(220, 220, 255));
                        frame.set_frame(FrameType::FlatBox);
                        continue;
                    }

                    if col == 0 {
                        let row_index = current_row_val + row - 1;
                        frame.set_label(&(row_index + 1).to_string());
                        frame.set_label_font(Font::HelveticaBold);
                        frame.set_color(Color::from_rgb(220, 220, 255));
                        frame.set_frame(FrameType::FlatBox);
                        continue;
                    }

                    // Highlight selected cell
                    if let (Some(sel_row), Some(sel_col)) = *selected_cell {
                        if row == sel_row as usize && col == sel_col as usize {
                            frame.set_color(Color::from_rgb(200, 230, 255));
                            frame.set_frame(FrameType::FlatBox);
                        }
                    }

                    // Display cell content
                    let actual_row = *current_row.lock().unwrap() + row - 1;
                    let actual_col = *current_col.lock().unwrap() + col - 1;

                    // println!("current_row: {}, current_col: {}", current_row_val, current_col_val);
                    if row > 0 && col > 0 {
                        if actual_row < sheet_data.rows as usize
                            && actual_col < sheet_data.columns as usize
                        {
                            let cell = &sheet_data.all_cells[actual_row][actual_col];
                            let cell_display = format!("{:9}", cell.value);

                            // Apply formatting based on cell properties
                            if cell.is_bold && cell.is_italics {
                                frame.set_label(&cell_display);
                                frame.set_label_font(Font::HelveticaBoldItalic);
                            } else if cell.is_bold {
                                frame.set_label(&cell_display);
                                frame.set_label_font(Font::HelveticaBold);
                            } else if cell.is_italics {
                                frame.set_label(&cell_display);
                                frame.set_label_font(Font::HelveticaItalic);
                            } else {
                                frame.set_label(&cell_display);
                            }

                            if cell.is_error {
                                frame.set_label("ERR");
                                frame.set_color(Color::Red);
                            }
                        } else {
                            frame.set_label(""); // Out of bounds
                        }
                    }
                    frame.redraw();
                }
            }

            // Check for errors in STATUS_EXTENSION
            let mut status = status_extension.lock().unwrap();
            if *status == 1 {
                alert(200, 200, "Invalid input!");
                *status = 0;
            }
            if *status == 2 {
                alert(200, 200, "division by zero!");
                *status = 0;
            } else if *status == 3 {
                alert(200, 200, "cyclic dependence found");
                *status = 0;
            } else if *status > 3 {
                alert(200, 200, "error occured!");
                *status = 0;
            }

            // prev_data = sheet_data;
        });

        app.run().unwrap();
    });
}
