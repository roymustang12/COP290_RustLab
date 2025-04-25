use plotters::prelude::*;
use plotters::style::{BLACK, BLUE, RED, WHITE};
const GRAY: RGBColor = RGBColor(169, 169, 169);

/// Plots a histogram for the given data and saves it to a file.
///
/// # Arguments
/// * `data` - A slice of `f64` values representing the data to plot.
/// * `filename` - The name of the file to save the histogram to.
///
/// # Returns
/// * `Ok(())` if the histogram is successfully generated.
/// * `Err` if an error occurs (e.g., empty data or file write failure).
///
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

    chart.draw_series(frequencies.iter().enumerate().map(|(i, &count)| {
        Rectangle::new([(i, 0), (i + 1, count)], BLUE.filled().stroke_width(1))
    }))?;

    root.present()?; // Save the file
    Ok(())
}

/// Plots a line graph for the given data and saves it to a file.
///
/// # Arguments
/// * `data` - A slice of `f64` values representing the data to plot.
/// * `filename` - The name of the file to save the line graph to.
///
/// # Returns
/// * `Ok(())` if the line graph is successfully generated.
/// * `Err` if an error occurs (e.g., empty data or file write failure).
pub fn plot_line(data: &[Vec<f64>], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if data.is_empty() || data.iter().all(|col| col.is_empty()) {
        return Err("No data to plot".into());
    }

    // Create a drawing area
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    // Determine maximum length (x range) and y value (y range)
    let x_max = data.iter().map(|col| col.len()).max().unwrap_or(0);
    let y_max = data
        .iter()
        .flat_map(|col| col.iter())
        .cloned()
        .fold(f64::NAN, f64::max)
        .max(1.0);

    // Create a chart and set the range for x and y axes
    let mut chart = ChartBuilder::on(&root)
        .caption("Line Plot", ("sans-serif", 30))
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        // Use Range instead of RangeInclusive for the x-axis
        .build_cartesian_2d(0..x_max as i32, 0f64..y_max)?;

    chart.configure_mesh().draw()?;

    let colors = [&BLUE, &RED, &GREEN, &CYAN, &MAGENTA, &BLACK, &YELLOW, &GRAY];

    // Plot each column as a line series
    for (i, column) in data.iter().enumerate() {
        let color = colors[i % colors.len()];

        // Draw the line series for each column
        chart
            .draw_series(LineSeries::new(
                column.iter().enumerate().map(|(x, y)| (x as i32, *y)),
                color,
            ))?
            .label(format!("Col {}", i + 1))
            .legend(move |(x, y)| PathElement::new([(x, y), (x + 20, y)], color));
    }

    // Configure and draw the legend
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

/// Plots a scatter plot for the given x and y data and saves it to a file.
///
/// # Arguments
/// * `x` - A slice of `f64` values representing the x-coordinates.
/// * `y` - A slice of `f64` values representing the y-coordinates.
/// * `filename` - The name of the file to save the scatter plot to.
///
/// # Returns
/// * `Ok(())` if the scatter plot is successfully generated.
/// * `Err` if an error occurs (e.g., mismatched lengths of x and y data or file write failure).
///
pub fn plot_scatter(
    x: &[f64],
    y: &[f64],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
        x.iter()
            .zip(y.iter())
            .map(|(&x, &y)| Circle::new((x, y), 4, RED.filled())),
    )?;

    Ok(())
}
