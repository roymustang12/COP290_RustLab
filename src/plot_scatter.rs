use plotters::prelude::*;

pub fn plot_scatter(x: &[f64], y: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if x.len() != y.len() {
        return Err("x and y vectors must be the same length".into());
    }

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = *x.iter().cloned().fold(f64::NAN, f64::min)..*x.iter().cloned().fold(f64::NAN, f64::max);
    let y_range = *y.iter().cloned().fold(f64::NAN, f64::min)..*y.iter().cloned().fold(f64::NAN, f64::max);

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
