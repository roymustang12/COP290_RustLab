use plotters::prelude::*;

pub fn plot_histogram(data: &[f64], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bins = 10;
    let min = *data.iter().cloned().fold(f64::NAN, f64::min);
    let max = *data.iter().cloned().fold(f64::NAN, f64::max);
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
        .caption("Histogram", ("sans-serif", 30))
        .margin(10)
        .build_cartesian_2d(0..bins, 0..max_count)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        frequencies.iter().enumerate().map(|(i, &count)| {
            Rectangle::new([(i, 0), (i + 1, count)], BLUE.filled())
        }),
    )?;

    Ok(())
}
