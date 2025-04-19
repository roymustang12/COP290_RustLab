use plotters::prelude::*;
use plotters::style::{BLUE, RED, WHITE};
use plotters::coord::types::RangedCoordf64;

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

    // Add bin labels
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    root.present()?; // Ensure the file is saved
    Ok(())
}



pub fn plot_line(data: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_y = data.iter().cloned().fold(f64::NAN, f64::max).max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("Line Plot", ("sans-serif", 30))
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .build_cartesian_2d(0..data.len(), 0f64..max_y)?;

    chart.configure_mesh().draw()?;

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

    let x_range = x.iter().cloned().fold(f64::NAN, f64::min)..x.iter().cloned().fold(f64::NAN, f64::max);
    let y_range = y.iter().cloned().fold(f64::NAN, f64::min)..y.iter().cloned().fold(f64::NAN, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Scatter Plot", ("sans-serif", 30))
        .margin(10)
        .build_cartesian_2d(x_range, y_range)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        x.iter().zip(y.iter()).map(|(&x, &y)| Circle::new((x, y), 4, RED.filled())),
    )?;

    Ok(())
}