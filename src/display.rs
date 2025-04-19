// use fltk::{
//     app,
//     enums::{Align, Color, FrameType},
//     frame::Frame,
//     prelude::*,
//     window::Window,
// };
// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::Duration;

// const ROWS: usize = 11;
// const COLS: usize = 11;
// const CELL_WIDTH: i32 = 100;
// const CELL_HEIGHT: i32 = 30;

// pub fn launch_gui(data: Arc<Mutex<Vec<Vec<String>>>>) {
//     thread::spawn(move || {
//         let app = app::App::default();
//         let win_w = (COLS as i32) * CELL_WIDTH;
//         let win_h = (ROWS as i32) * CELL_HEIGHT;
//         let mut wind = Window::new(100, 100, win_w + 20, win_h + 20, "Rust Spreadsheet Display");

//         let mut frames: Vec<Vec<Frame>> = vec![];
//         for row in 0..ROWS {
//             let mut row_frames = vec![];
//             for col in 0..COLS {
//                 let cell_x = col as i32 * CELL_WIDTH + 10;
//                 let cell_y = row as i32 * CELL_HEIGHT + 10;
//                 let mut frame = Frame::new(cell_x, cell_y, CELL_WIDTH, CELL_HEIGHT, "");
//                 frame.set_label_size(14);
//                 frame.set_align(Align::Inside | Align::Left);
//                 row_frames.push(frame);
//             }
//             frames.push(row_frames);
//         }
        
//         wind.end();
//         wind.show();

//         let mut prev_data = vec![vec![String::new(); COLS]; ROWS];

//         app::add_idle3(move |_| {
//             let new_data = data.lock().unwrap().clone();

//             for row in 0..ROWS {
//                 for col in 0..COLS {
//                     if new_data.get(row).and_then(|r| r.get(col)) != prev_data.get(row).and_then(|r| r.get(col)) {
//                         let label = new_data.get(row).and_then(|r| r.get(col)).map_or("", |v| v);
//                         let frame = &mut frames[row][col];
//                         frame.set_label(label);
//                         if label == "ERR" {
//                             frame.set_color(Color::Red);
//                             frame.set_frame(FrameType::FlatBox);
//                         } else if row == 0 || col == 0 {
//                             frame.set_color(Color::from_rgb(220, 220, 255));
//                             frame.set_frame(FrameType::FlatBox);
//                         } else {
//                             frame.set_color(Color::White);
//                             frame.set_frame(FrameType::DownBox);
//                         }
//                         frame.redraw();
//                     }
//                 }
//             }

//             prev_data = new_data;
//         });

//         app.run().unwrap();
//     });
// }
