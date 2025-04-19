
pub fn linear_regression(x_values: &[f64], y_values: &[f64]) -> (f64, f64) {
    
    if x_values.len() < 2 || x_values.len() != y_values.len() {
        return (0.0, 0.0); 
    }

    let n = x_values.len() as f64;
    
    
    let mean_x: f64 = x_values.iter().map(|&x| x as f64).sum::<f64>() / n;
    let mean_y: f64 = y_values.iter().map(|&y| y as f64).sum::<f64>() / n;
    
    
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    
    for i in 0..x_values.len() {
        let x_diff = (x_values[i] as f64) - mean_x;
        let y_diff = (y_values[i] as f64) - mean_y;
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


pub fn forecast(x: f64, x_values: &[f64], y_values: &[f64]) -> f64 {
    let (slope, intercept) = linear_regression(x_values, y_values);
    (intercept + slope * (x)) 
}



