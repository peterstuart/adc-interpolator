#![cfg_attr(not(test), no_std)]

mod interpolate;

use interpolate::interpolate;

/// Converts a voltage and corresponding value into a pair of `(adc_value, value)`.
///
/// Use to create a table to be used by an [AdcInterpolator](AdcInterpolator).
///
/// # Arguments
///
/// - `max_voltage`: The voltage corresponding to the largest value possible for the ADC (mV)
/// - `precision`: The precision of the ADC in bits (eg. for 10-bit precision, use `10`)
/// - `voltage`: The voltage to convert (mV)
/// - `value`: The value to use in the pair
pub const fn pair(max_voltage: u32, precision: u32, voltage: u32, value: u32) -> (u32, u32) {
    let max_adc_value = 2u32.pow(precision);
    let adc_value = voltage * max_adc_value / max_voltage;

    (adc_value, value)
}

#[derive(Debug)]
pub struct AdcInterpolator<const LENGTH: usize> {
    table: [(u32, u32); LENGTH],
}

impl<const LENGTH: usize> AdcInterpolator<LENGTH> {
    /// Returns an interpolator using the provided table.
    ///
    /// The values in the table *must* be in ascending order by
    /// voltage. (ie. If you are using [`pair`](pair) to create the
    /// pairs in the table, the `voltage` parameter must increase with
    /// each pair.)
    ///
    /// # Examples
    ///
    /// Use [`pair`](pair) to create the pairs in `table`:
    ///
    /// ```
    /// use adc_interpolator::{AdcInterpolator, pair};
    ///
    /// AdcInterpolator::new([
    ///   pair(1000, 12, 100, 40),
    ///   pair(1000, 12, 200, 30),
    ///   pair(1000, 12, 300, 10),
    /// ]);
    pub const fn new(table: [(u32, u32); LENGTH]) -> Self {
        Self { table }
    }

    /// Returns a value based on the table, using linear interpolation
    /// between values in the table if necessary. If `adc_value` falls
    /// outside the range of the table, returns `None`.
    pub fn value(&self, adc_value: u32) -> Option<u32> {
        let (x0, y0, x1, y1) = self
            .table
            .iter()
            .enumerate()
            .find_map(|(index, (x0, y0))| {
                let (x1, y1) = self.table.get(index + 1)?;

                if adc_value >= *x0 && adc_value <= *x1 {
                    Some((x0, y0, x1, y1))
                } else {
                    None
                }
            })?;

        Some(interpolate(*x0, *x1, *y0, *y1, adc_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: AdcInterpolator<3> = AdcInterpolator::new([
        pair(1000, 12, 100, 40),
        pair(1000, 12, 200, 30),
        pair(1000, 12, 300, 10),
    ]);

    #[test]
    fn matching_exact_values() {
        assert_eq!(TABLE.value(409), Some(40));
        assert_eq!(TABLE.value(819), Some(30));
        assert_eq!(TABLE.value(1228), Some(10));
    }

    #[test]
    fn interpolates() {
        assert_eq!(TABLE.value(502), Some(38));
        assert_eq!(TABLE.value(614), Some(35));
        assert_eq!(TABLE.value(1023), Some(21));
    }

    #[test]
    fn outside_range() {
        assert_eq!(TABLE.value(0), None);
        assert_eq!(TABLE.value(408), None);
        assert_eq!(TABLE.value(1229), None);
        assert_eq!(TABLE.value(10000), None);
    }
}
