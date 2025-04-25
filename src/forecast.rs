/// Performs linear regression on a set of data points.
///
/// This function calculates the slope and intercept of the best-fit line
/// for the given `x_values` and `y_values` using the least squares method.
///
/// # Arguments
/// * `x_values` - A slice of `f64` values representing the x-coordinates of the data points.
/// * `y_values` - A slice of `f64` values representing the y-coordinates of the data points.
///
/// # Returns
/// * `(f64, f64)` - A tuple containing the slope and intercept of the best-fit line.
///   If the input data is invalid (e.g., fewer than 2 points or all x-values are the same),
///   the function returns `(0.0, 0.0)`.
pub fn linear_regression(x_values: &[f64], y_values: &[f64]) -> (f64, f64) {
    if x_values.len() < 2 || x_values.len() != y_values.len() {
        return (0.0, 0.0);
    }

    let n = x_values.len() as f64;

    let mean_x: f64 = x_values.iter().copied().sum::<f64>() / n;
    let mean_y: f64 = y_values.iter().copied().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 0..x_values.len() {
        let x_diff = x_values[i] - mean_x;
        let y_diff = y_values[i] - mean_y;
        numerator += x_diff * y_diff;
        denominator += x_diff * x_diff;
    }

    if denominator == 0.0 {
        return (0.0, 0.0);
    }

    let slope = numerator / denominator;
    let intercept = mean_y - slope * mean_x;

    (slope, intercept)
}

/// Predicts the y-value for a given x-value using linear regression.
///
/// This function uses the slope and intercept calculated by the `linear_regression`
/// function to predict the y-value for the given x-coordinate.
///
/// # Arguments
/// * `x` - The x-coordinate for which the y-value is to be predicted.
/// * `x_values` - A slice of `f64` values representing the x-coordinates of the data points.
/// * `y_values` - A slice of `f64` values representing the y-coordinates of the data points.
pub fn forecast(x: f64, x_values: &[f64], y_values: &[f64]) -> f64 {
    let (slope, intercept) = linear_regression(x_values, y_values);
    intercept + slope * (x)
}
