use plotters::prelude::*;
use plotters::style::{BLACK, BLUE, RED, WHITE};
use plotters::coord::types::RangedCoordf64;

/// Helper to generate cell names like A1, A2, ..., from index
fn generate_cell_names(start_row: usize, col: usize, count: usize) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..count {
        let col_char = ((b'A' + col as u8) as char).to_string(); // Simple case: single letter col
        names.push(format!("{}{}", col_char, start_row + i + 1));
    }
    names
}

pub fn plot_histogram(data: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("No data provided for histogram".into());
    }

    let bins = 10;
    let min = data.iter().cloned().fold(f64::NAN, f64::min);
    let max = data.iter().cloned().fold(f64::NAN, f64::max);
    let bin_width = (max - min) / bins as f64;

    let mut frequencies = vec![0; bins];
    for &val in data {
        let index = ((val - min) / bin_width).floor() as usize;
        let index = index.min(bins - 1);
        frequencies[index] += 1;
    }

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = *frequencies.iter().max().unwrap_or(&1);

    let mut chart = ChartBuilder::on(&root)
        .caption("Histogram of Data", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..bins, 0..max_count)?;

    chart
        .configure_mesh()
        .x_desc("Bins")
        .y_desc("Frequency")
        .label_style(("sans-serif", 12))
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        frequencies.iter().enumerate().map(|(i, &count)| {
            Rectangle::new(
                [(i, 0), (i + 1, count)],
                BLUE.filled().stroke_width(1),
            )
        }),
    )?;

    root.present()?; // Save the file
    Ok(())
}

pub fn plot_line(data: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("No data to plot".into());
    }

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_y = data.iter().cloned().fold(f64::NAN, f64::max).ceil().max(1.0);
    let y_range = 0f64..max_y;
    let x_range = 0..data.len();

    let cell_names = generate_cell_names(0, 0, data.len());

    let mut chart = ChartBuilder::on(&root)
        .caption("Line Plot", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range.clone(), y_range.clone())?;

    chart
        .configure_mesh()
        .x_desc("Cell Index")
        .y_desc("Cell Value")
        .x_labels(data.len())
        .y_labels(max_y as usize + 1)
        .label_style(("sans-serif", 12))
        .axis_desc_style(("sans-serif", 15))
        .x_label_formatter(&|x| cell_names.get(*x).unwrap_or(&"".to_string()).to_string())
        .y_label_formatter(&|y| format!("{:.0}", y))
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().enumerate().map(|(i, v)| (i, *v)),
        &BLUE,
    ))?;

    Ok(())
}

pub fn plot_scatter(x: &[f64], y: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if x.len() != y.len() {
        return Err("x and y vectors must be the same length".into());
    }

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min).floor();
    let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max).ceil();
    let y_min = y.iter().cloned().fold(f64::INFINITY, f64::min).floor();
    let y_max = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max).ceil();

    let x_range = x_min..x_max;
    let y_range = y_min..y_max;

    let mut chart = ChartBuilder::on(&root)
        .caption("Scatter Plot", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range.clone(), y_range.clone())?;

    chart
        .configure_mesh()
        .x_desc("X Values")
        .y_desc("Y Values")
        .x_labels((x_max - x_min) as usize + 1)
        .y_labels((y_max - y_min) as usize + 1)
        .label_style(("sans-serif", 12))
        .axis_desc_style(("sans-serif", 15))
        .y_label_formatter(&|y| format!("{:.0}", y))
        .x_label_formatter(&|x| format!("{:.0}", x))
        .draw()?;

    chart.draw_series(
        x.iter().zip(y.iter()).map(|(&x, &y)| Circle::new((x, y), 4, RED.filled())),
    )?;

    Ok(())
}
