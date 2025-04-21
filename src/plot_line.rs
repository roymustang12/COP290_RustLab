pub fn plot_line(data: &[Vec<f64>], filename: &str, start_row: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Check if data is empty or not
    if data.is_empty() || data.iter().all(|col| col.is_empty()) {
        return Err("No data to plot".into());
    }

    // Create the drawing area
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate x and y range
    let x_max = data.iter().map(|col| col.len()).max().unwrap_or(0);
    let y_max = data
        .iter()
        .flat_map(|col| col.iter())
        .cloned()
        .fold(f64::NAN, f64::max)
        .max(1.0);

    // Create the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Line Plot", ("sans-serif", 30))
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0..x_max as i32, 0f64..y_max)?;

    // Configure the mesh and labels
    chart.configure_mesh().draw()?;

    // Define line colors for different columns
    let colors = [
        &BLUE, &RED, &GREEN, &CYAN, &MAGENTA, &BLACK, &YELLOW, &GRAY,
    ];

    // Plot the data for each column
    for (i, column) in data.iter().enumerate() {
        let color = colors[i % colors.len()];

        // Draw each column's data as a line series
        chart.draw_series(LineSeries::new(
            column.iter().enumerate().map(|(x, y)| (x as i32 + start_row, *y)),
            color,
        ))?
        .label(format!("Col {}", i + 1))
        .legend(move |(x, y)| PathElement::new([(x, y), (x + 20, y)], color));
    }

    // Draw the legend
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
