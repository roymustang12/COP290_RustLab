use plotters::prelude::*;

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
