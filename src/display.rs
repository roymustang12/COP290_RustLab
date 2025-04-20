use fltk::{
    app,
    enums::{Align, Color, FrameType, Font},
    frame::Frame,
    prelude::*,
    window::Window,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// The number of rows in the spreadsheet display.
const ROWS: usize = 11;

/// The number of columns in the spreadsheet display.
const COLS: usize = 11;

/// The width of each cell in the spreadsheet display (in pixels).
const CELL_WIDTH: i32 = 100;

/// The height of each cell in the spreadsheet display (in pixels).
const CELL_HEIGHT: i32 = 30;

/// Launches the graphical user interface (GUI) for the spreadsheet.
///
/// This function creates a window and displays the spreadsheet in a grid format.
/// It dynamically updates the display based on the data provided.
///
/// # Arguments
/// * `data` - A thread-safe shared reference to the spreadsheet data. Each cell's content is represented as a `String`.
///
/// # Behavior
/// * The GUI is displayed in a grid format with rows and columns.
/// * The content of each cell is updated dynamically based on the `data`.
/// * Styles such as bold, italics, and underline are applied based on specific markers in the cell content.
///
/// # Example
/// ```rust
/// let shared_data = Arc::new(Mutex::new(vec![vec!["A1".to_string(), "B1".to_string()], vec!["A2".to_string(), "B2".to_string()]]));
/// launch_gui(shared_data);
/// ```

pub fn launch_gui(data: Arc<Mutex<Vec<Vec<String>>>>) {
    thread::spawn(move || {
        let app = app::App::default();
        let win_w = (COLS as i32) * CELL_WIDTH;
        let win_h = (ROWS as i32) * CELL_HEIGHT;
        let mut wind = Window::new(100, 100, win_w + 20, win_h + 20, "Rust Spreadsheet Display");


         // Create a grid of frames to represent the spreadsheet cells
        let mut frames: Vec<Vec<Frame>> = vec![];
        for row in 0..ROWS {
            let mut row_frames = vec![];
            for col in 0..COLS {
                let cell_x = col as i32 * CELL_WIDTH + 10;
                let cell_y = row as i32 * CELL_HEIGHT + 10;
                let mut frame = Frame::new(cell_x, cell_y, CELL_WIDTH, CELL_HEIGHT, "");
                frame.set_label_size(14);
                frame.set_align(Align::Inside | Align::Left);
                row_frames.push(frame);
            }
            frames.push(row_frames);
        }

        wind.end();
        wind.show();

         // Initialize previous data for comparison

        let mut prev_data = vec![vec![String::new(); COLS]; ROWS];

        // Add an idle callback to update the GUI dynamically
        app::add_idle3(move |_| {
            let new_data = data.lock().unwrap().clone();

            for row in 0..ROWS {
                for col in 0..COLS {
                    if new_data.get(row).and_then(|r| r.get(col)) != prev_data.get(row).and_then(|r| r.get(col)) {
                        let label = new_data.get(row).and_then(|r| r.get(col)).map_or("", |v| v);
                        let frame = &mut frames[row][col];

                        // Apply styles based on the label content
                        if label.starts_with("*") && label.ends_with("*") {
                            // Bold
                            frame.set_label(&label[1..label.len() - 1]); // Remove the * markers
                            frame.set_label_font(Font::HelveticaBold);
                        } else if label.starts_with("_") && label.ends_with("_") {
                            // Italics
                            frame.set_label(&label[1..label.len() - 1]); // Remove the _ markers
                            frame.set_label_font(Font::HelveticaItalic);
                        } else if label.starts_with("~") && label.ends_with("~") {
                            // BoldItalics (simulated by adding a marker)
                            frame.set_label(&label[1..label.len() - 1]); // Remove the ~ markers
                            frame.set_label_font(Font::HelveticaBoldItalic); // Default font
                            frame.set_frame(FrameType::FlatBox); // Simulate underline
                        } else {
                            frame.set_label(label);
                            frame.set_label_font(Font::Helvetica); // Default font
                        }

                        // Set cell background color
                        if label == "ERR" {
                            frame.set_color(Color::Red);
                            frame.set_frame(FrameType::FlatBox);
                        } else if row == 0 || col == 0 {
                            frame.set_color(Color::from_rgb(220, 220, 255));
                            frame.set_frame(FrameType::FlatBox);
                        } else {
                            frame.set_color(Color::White);
                            frame.set_frame(FrameType::DownBox);
                        }
                        frame.redraw();
                    }
                }
            }

            prev_data = new_data;
        });

        app.run().unwrap();
    });
}