use alloy_sol_types::sol;
use primitive_types::U256;

/// Represents a time series with timestamps and corresponding values.
#[derive(Clone, Debug)]
pub struct TimeSeries {
    pub timestamps: Vec<u64>,
    pub values: Vec<f64>,
}

impl TimeSeries {
    /// Creates a new TimeSeries instance.
    ///
    /// # Arguments
    /// * `timestamps` - A vector of Unix timestamps
    /// * `values` - A vector of corresponding values
    ///
    /// # Panics
    /// Panics if the lengths of timestamps and values are not equal.
    pub fn new(timestamps: Vec<u64>, values: Vec<f64>) -> Self {
        assert_eq!(
            timestamps.len(),
            values.len(),
            "Timestamps and values must have the same length"
        );
        TimeSeries { timestamps, values }
    }

    /// Calculates the mean of the time series values.
    pub fn mean(&self) -> f64 {
        let sum: f64 = self.values.iter().sum();
        sum / self.values.len() as f64
    }

    /// Calculates the median of the time series values.
    pub fn median(&self) -> f64 {
        let mut sorted_values = self.values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted_values.len() / 2;
        if sorted_values.len() % 2 == 0 {
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[mid]
        }
    }

    /// Calculates the standard deviation of the time series values.
    pub fn std_dev(&self) -> f64 {
        let mean = self.mean();
        let variance: f64 = self
            .values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>()
            / self.values.len() as f64;
        variance.sqrt()
    }

    /// Computes the moving average of the time series.
    ///
    /// # Arguments
    /// * `window_size` - The size of the moving window
    pub fn moving_average(&self, window_size: usize) -> TimeSeries {
        let mut ma_values = Vec::with_capacity(self.values.len());
        for i in 0..self.values.len() {
            let start = if i < window_size {
                0
            } else {
                i - window_size + 1
            };
            let window = &self.values[start..=i];
            let avg = window.iter().sum::<f64>() / window.len() as f64;
            ma_values.push(avg);
        }
        TimeSeries::new(self.timestamps.clone(), ma_values)
    }

    /// Computes the exponential moving average of the time series.
    ///
    /// # Arguments
    /// * `alpha` - The smoothing factor (0 < alpha <= 1)
    pub fn exponential_moving_average(&self, alpha: f64) -> TimeSeries {
        assert!(
            (0.0..=1.0).contains(&alpha),
            "Alpha must be between 0 and 1"
        );
        let mut ema_values = Vec::with_capacity(self.values.len());
        ema_values.push(self.values[0]);
        for i in 1..self.values.len() {
            let ema = alpha * self.values[i] + (1.0 - alpha) * ema_values[i - 1];
            ema_values.push(ema);
        }
        TimeSeries::new(self.timestamps.clone(), ema_values)
    }

    /// Performs simple exponential smoothing for forecasting.
    ///
    /// # Arguments
    /// * `alpha` - The smoothing factor (0 < alpha <= 1)
    /// * `horizon` - The number of time steps to forecast
    pub fn simple_exponential_smoothing(&self, alpha: f64, horizon: usize) -> TimeSeries {
        assert!(
            (0.0..=1.0).contains(&alpha),
            "Alpha must be between 0 and 1"
        );
        let mut forecast = Vec::with_capacity(self.values.len() + horizon);
        forecast.push(self.values[0]);
        for i in 1..self.values.len() {
            let smooth = alpha * self.values[i] + (1.0 - alpha) * forecast[i - 1];
            forecast.push(smooth);
        }
        for _ in 0..horizon {
            forecast.push(*forecast.last().unwrap());
        }
        let mut timestamps = self.timestamps.clone();
        let last_timestamp = *timestamps.last().unwrap();
        let time_step = if timestamps.len() > 1 {
            timestamps[1] - timestamps[0]
        } else {
            1
        };
        for i in 1..=horizon {
            timestamps.push(last_timestamp + i as u64 * time_step);
        }
        TimeSeries::new(timestamps, forecast)
    }
}

sol! {
    /// Defines the structure for public values output by the ZK proof.
    struct PublicValuesStruct {
        uint64[] timestamps;
        uint256[] forecast_values;
        uint256 mean;
        uint256 std_dev;
    }
}

/// Converts an f64 to a U256 for Solidity compatibility.
///
/// This function multiplies the f64 by 1e18 and converts it to a U256.
/// This allows for 18 decimal places of precision in Solidity.
pub fn f64_to_u256(value: f64) -> U256 {
    U256::from_dec_str(&format!("{:.0}", value.abs() * 1e18)).unwrap()
}

/// Converts a Vec<f64> to a Vec<U256> for Solidity compatibility.
pub fn vec_f64_to_u256(values: &[f64]) -> Vec<U256> {
    values.iter().map(|&v| f64_to_u256(v)).collect()
}

/// Converts a U256 back to an f64.
///
/// This function is the inverse of f64_to_u256.
pub fn u256_to_f64(value: U256) -> f64 {
    value.to_string().parse::<f64>().unwrap() / 1e18
}

/// Converts a Vec<U256> back to a Vec<f64>.
pub fn vec_u256_to_f64(values: &[U256]) -> Vec<f64> {
    values.iter().map(|&v| u256_to_f64(v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_creation() {
        let ts = TimeSeries::new(vec![1, 2, 3], vec![1.0, 2.0, 3.0]);
        assert_eq!(ts.timestamps, vec![1, 2, 3]);
        assert_eq!(ts.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_mean() {
        let ts = TimeSeries::new(vec![1, 2, 3], vec![1.0, 2.0, 3.0]);
        assert_eq!(ts.mean(), 2.0);
    }

    #[test]
    fn test_median() {
        let ts = TimeSeries::new(vec![1, 2, 3, 4], vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(ts.median(), 2.5);
    }

    #[test]
    fn test_std_dev() {
        let ts = TimeSeries::new(vec![1, 2, 3], vec![1.0, 2.0, 3.0]);
        assert!((ts.std_dev() - 0.816496580927726).abs() < 1e-10);
    }

    #[test]
    fn test_moving_average() {
        let ts = TimeSeries::new(vec![1, 2, 3, 4, 5], vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let ma = ts.moving_average(3);
        assert_eq!(ma.values, vec![1.0, 1.5, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_exponential_moving_average() {
        let ts = TimeSeries::new(vec![1, 2, 3, 4, 5], vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let ema = ts.exponential_moving_average(0.5);
        assert_eq!(ema.values[0], 1.0);
        assert!((ema.values[4] - 3.9375).abs() < 1e-10);
    }

    #[test]
    fn test_simple_exponential_smoothing() {
        let ts = TimeSeries::new(vec![1, 2, 3, 4, 5], vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let ses = ts.simple_exponential_smoothing(0.5, 2);
        assert_eq!(ses.timestamps, vec![1, 2, 3, 4, 5, 6, 7]);
        assert!((ses.values[6] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_f64_to_u256_conversion() {
        let value = std::f64::consts::PI;
        let converted = f64_to_u256(value);
        let back = u256_to_f64(converted);
        assert!((value - back).abs() < 1e-10);
    }
}
