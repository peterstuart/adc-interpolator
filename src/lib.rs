#![cfg_attr(not(test), no_std)]

mod interpolate;

use embedded_hal::adc::{Channel, OneShot};
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
///
/// # Examples
///
/// ```
/// use adc_interpolator::pair;
///
/// pair(
///     3300, // 3.3 V max voltage
///     10,   // 10 bits of precision
///     420,  // 0.42 V
///     80,   // value
/// );
/// ```
pub const fn pair(max_voltage: u32, precision: u32, voltage: u32, value: u32) -> (u32, u32) {
    let max_adc_value = 2u32.pow(precision);
    let adc_value = voltage * max_adc_value / max_voltage;

    (adc_value, value)
}

#[derive(Debug)]
pub struct AdcInterpolator<Adc, Pin, const LENGTH: usize> {
    adc: Adc,
    pin: Pin,
    table: [(u32, u32); LENGTH],
}

impl<Adc, Pin, const LENGTH: usize> AdcInterpolator<Adc, Pin, LENGTH> {
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
    pub fn new<ADC, Word>(adc: Adc, pin: Pin, table: [(u32, u32); LENGTH]) -> Self
    where
        Word: Into<u32>,
        Pin: Channel<ADC>,
        Adc: OneShot<ADC, Word, Pin>,
    {
        debug_assert!(
            table.windows(2).all(|w| w[0].0 <= w[1].0),
            "The values in `table` must be in ascending order by voltage"
        );

        Self { adc, pin, table }
    }

    pub fn free(self) -> (Adc, Pin) {
        (self.adc, self.pin)
    }

    /// Returns a value based on the table, using linear interpolation
    /// between values in the table if necessary. If `adc_value` falls
    /// outside the range of the table, returns `None`.
    pub fn read<ADC, Word>(
        &mut self,
    ) -> Result<Option<u32>, nb::Error<<Adc as OneShot<ADC, Word, Pin>>::Error>>
    where
        Word: Into<u32>,
        Pin: Channel<ADC>,
        Adc: OneShot<ADC, Word, Pin>,
    {
        let adc_value: u32 = self.adc.read(&mut self.pin)?.into();

        let result = self.table.iter().enumerate().find_map(|(index, (x0, y0))| {
            let (x1, y1) = self.table.get(index + 1)?;

            if adc_value >= *x0 && adc_value <= *x1 {
                Some((x0, y0, x1, y1))
            } else {
                None
            }
        });

        Ok(result.map(|(x0, y0, x1, y1)| interpolate(*x0, *x1, *y0, *y1, adc_value)))
    }

    /// Returns the smallest value that can be returned by
    /// [`read`](AdcInterpolator::read).
    pub fn min_value(&self) -> u32 {
        self.first_value().min(self.last_value())
    }

    /// Returns the largest value that can be returned by
    /// [`read`](AdcInterpolator::read).
    pub fn max_value(&self) -> u32 {
        self.first_value().max(self.last_value())
    }

    fn first_value(&self) -> u32 {
        self.table.first().unwrap().1
    }

    fn last_value(&self) -> u32 {
        self.table.last().unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::{
        adc::{Mock, MockChan0, Transaction},
        common::Generic,
        MockError,
    };
    use std::io::ErrorKind;

    const TABLE_POSITIVE: [(u32, u32); 3] = [
        pair(1000, 12, 100, 10),
        pair(1000, 12, 200, 30),
        pair(1000, 12, 300, 40),
    ];

    const TABLE_NEGATIVE: [(u32, u32); 3] = [
        pair(1000, 12, 100, 40),
        pair(1000, 12, 200, 30),
        pair(1000, 12, 300, 10),
    ];

    const TABLE_INVALID: [(u32, u32); 3] = [
        pair(1000, 12, 300, 40),
        pair(1000, 12, 200, 30),
        pair(1000, 12, 100, 10),
    ];

    pub fn interpolator<const LENGTH: usize>(
        table: [(u32, u32); LENGTH],
        expectations: &[Transaction<u32>],
    ) -> AdcInterpolator<Generic<Transaction<u32>>, MockChan0, LENGTH> {
        let adc = Mock::new(expectations);
        let pin = MockChan0 {};

        AdcInterpolator::new(adc, pin, table)
    }

    fn assert_read_ok<const LENGTH: usize>(
        table: [(u32, u32); LENGTH],
        value: u32,
        expected: Option<u32>,
    ) {
        let expectations = [Transaction::read(0, value)];
        assert_eq!(interpolator(table, &expectations).read(), Ok(expected))
    }

    #[test]
    #[should_panic]
    fn panics_if_unsorted_tabled() {
        interpolator(TABLE_INVALID, &[]);
    }

    #[test]
    fn matching_exact_values() {
        assert_read_ok(TABLE_NEGATIVE, 409, Some(40));
        assert_read_ok(TABLE_NEGATIVE, 819, Some(30));
        assert_read_ok(TABLE_NEGATIVE, 1228, Some(10));
    }

    #[test]
    fn interpolates() {
        assert_read_ok(TABLE_NEGATIVE, 502, Some(38));
        assert_read_ok(TABLE_NEGATIVE, 614, Some(35));
        assert_read_ok(TABLE_NEGATIVE, 1023, Some(21));
    }

    #[test]
    fn outside_range() {
        assert_read_ok(TABLE_NEGATIVE, 0, None);
        assert_read_ok(TABLE_NEGATIVE, 408, None);
        assert_read_ok(TABLE_NEGATIVE, 1229, None);
        assert_read_ok(TABLE_NEGATIVE, 10000, None);
    }

    #[test]
    fn error() {
        let expectations =
            [Transaction::read(0, 0).with_error(MockError::Io(ErrorKind::InvalidData))];
        assert!(interpolator(TABLE_POSITIVE, &expectations).read().is_err());
    }

    #[test]
    fn min_value() {
        assert_eq!(interpolator(TABLE_POSITIVE, &[]).min_value(), 10);
        assert_eq!(interpolator(TABLE_NEGATIVE, &[]).min_value(), 10);
    }

    #[test]
    fn max_value() {
        assert_eq!(interpolator(TABLE_POSITIVE, &[]).max_value(), 40);
        assert_eq!(interpolator(TABLE_NEGATIVE, &[]).max_value(), 40);
    }
}
