use crate::interpolate::interpolate;
use core::fmt;
use embedded_hal::adc::{Channel, OneShot};

/// Configuration for an `AdcInterpolator`.
///
/// - `max_voltage`: The voltage corresponding to the largest value possible for the ADC (mV)
/// - `precision`: The precision of the ADC in bits (eg. for 10-bit precision, use `10`)
/// - `voltage_to_values`: An array of tuples of `(voltage in mV, value)` which will be used for the interpolation
///
/// # Examples
///
/// ```
/// use adc_interpolator::Config;
///
/// let config = Config {
///     max_voltage: 3300, // 3.3 V
///     precision: 10,     // 10 bits of precision
///     voltage_to_values: [
///         (100, 5),   // 100 mV  -> 5
///         (500, 10),  // 500 mV  -> 10
///         (2000, 15), // 2000 mV -> 15
///     ],
/// };
/// ```
pub struct Config<const LENGTH: usize> {
    pub max_voltage: u32,
    pub precision: u32,
    pub voltage_to_values: [(u32, u32); LENGTH],
}

impl<const LENGTH: usize> Config<LENGTH> {
    fn table<Word>(&self) -> [(Word, u32); LENGTH]
    where
        Word: Copy + PartialOrd + TryFrom<u32>,
        <Word as TryFrom<u32>>::Error: fmt::Debug,
    {
        let mut table: [(Word, u32); LENGTH] = [(0.try_into().unwrap(), 0); LENGTH];

        for (index, (voltage, value)) in self.voltage_to_values.into_iter().enumerate() {
            let max_adc_value = 2u32.pow(self.precision);
            let adc_value = voltage * max_adc_value / self.max_voltage;

            table[index] = (adc_value.try_into().unwrap(), value);
        }

        table
    }
}

#[derive(Debug)]
pub struct AdcInterpolator<Pin, Word, const LENGTH: usize> {
    pin: Pin,
    table: [(Word, u32); LENGTH],
}

type Error<Adc, ADC, Word, Pin> = nb::Error<<Adc as OneShot<ADC, Word, Pin>>::Error>;

impl<Pin, Word, const LENGTH: usize> AdcInterpolator<Pin, Word, LENGTH> {
    /// Returns an interpolator using the provided `config`.
    ///
    /// The values in `config`'s `voltage_to_values` field must be in
    /// ascending order by voltage or this function will panic when
    /// running in debug mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use adc_interpolator::{AdcInterpolator, Config};
    /// # use embedded_hal_mock::{
    /// #     adc::{Mock, MockChan0, Transaction},
    /// #     common::Generic,
    /// #     MockError,
    /// # };
    /// #
    /// # let pin = MockChan0 {};
    ///
    /// let config = Config {
    ///     max_voltage: 1000,
    ///     precision: 12,
    ///     voltage_to_values: [
    ///         (100, 40),
    ///         (200, 30),
    ///         (300, 10),
    ///     ],
    /// };
    ///
    /// let interpolator = AdcInterpolator::new(pin, config);
    /// # let interpolator_u16: AdcInterpolator<MockChan0, u16, 3> = interpolator;
    pub fn new<ADC>(pin: Pin, config: Config<LENGTH>) -> Self
    where
        Word: Copy + PartialOrd + TryFrom<u32>,
        <Word as TryFrom<u32>>::Error: fmt::Debug,
        Pin: Channel<ADC>,
    {
        debug_assert!(
            config
                .voltage_to_values
                .windows(2)
                .all(|w| w[0].0 <= w[1].0),
            "The values in table must be in ascending order by voltage"
        );

        Self {
            pin,
            table: config.table(),
        }
    }

    /// Destroys the interpolator and returns the `Pin`.
    pub fn free(self) -> Pin {
        self.pin
    }

    /// Returns a value based on the table, using linear interpolation
    /// between values in the table if necessary. If `adc_value` falls
    /// outside the range of the table, returns `Ok(None)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use adc_interpolator::{AdcInterpolator, Config};
    /// # use embedded_hal_mock::{
    /// #     adc::{Mock, MockChan0, Transaction},
    /// #     common::Generic,
    /// #     MockError,
    /// # };
    /// #
    /// # let expectations: [Transaction<u16>; 1] = [Transaction::read(0, 614)];
    /// # let mut adc = Mock::new(&expectations);
    /// # let pin = MockChan0 {};
    ///
    /// let config = Config {
    ///     max_voltage: 1000,
    ///     precision: 12,
    ///     voltage_to_values: [
    ///         (100, 40),
    ///         (200, 30),
    ///         (300, 10),
    ///     ],
    /// };
    ///
    /// let mut interpolator = AdcInterpolator::new(pin, config);
    ///
    /// // With voltage at 150 mV, the value is 35
    /// assert_eq!(interpolator.read(&mut adc), Ok(Some(35)));
    /// ```
    pub fn read<Adc, ADC>(
        &mut self,
        adc: &mut Adc,
    ) -> Result<Option<u32>, Error<Adc, ADC, Word, Pin>>
    where
        Word: Copy + Into<u32> + PartialEq + PartialOrd,
        Pin: Channel<ADC>,
        Adc: OneShot<ADC, Word, Pin>,
    {
        let adc_value = adc.read(&mut self.pin)?;

        let result = self.table.iter().enumerate().find_map(|(index, (x0, y0))| {
            let (x1, y1) = self.table.get(index + 1)?;

            if adc_value >= *x0 && adc_value <= *x1 {
                Some((x0, y0, x1, y1))
            } else {
                None
            }
        });

        Ok(result.map(|(x0, y0, x1, y1)| {
            interpolate((*x0).into(), (*x1).into(), *y0, *y1, adc_value.into())
        }))
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

    fn table_positive() -> Config<3> {
        Config {
            max_voltage: 1000,
            precision: 12,
            voltage_to_values: [(100, 10), (200, 30), (300, 40)],
        }
    }

    fn table_negative() -> Config<3> {
        Config {
            max_voltage: 1000,
            precision: 12,
            voltage_to_values: [(100, 40), (200, 30), (300, 10)],
        }
    }

    fn table_invalid() -> Config<3> {
        Config {
            max_voltage: 1000,
            precision: 12,
            voltage_to_values: [(300, 40), (200, 30), (100, 10)],
        }
    }

    fn interpolator<const LENGTH: usize>(
        config: Config<LENGTH>,
    ) -> AdcInterpolator<MockChan0, u16, LENGTH> {
        let pin = MockChan0 {};
        AdcInterpolator::new(pin, config)
    }

    fn adc(expectations: &[Transaction<u16>]) -> Generic<Transaction<u16>> {
        Mock::new(expectations)
    }

    fn assert_read_ok<const LENGTH: usize>(
        config: Config<LENGTH>,
        value: u16,
        expected: Option<u32>,
    ) {
        let mut interpolator = interpolator(config);
        let expectations = [Transaction::read(0, value)];
        let mut adc = adc(&expectations);

        assert_eq!(interpolator.read(&mut adc), Ok(expected))
    }

    #[test]
    #[should_panic]
    fn panics_if_unsorted_tabled() {
        interpolator(table_invalid());
    }

    #[test]
    fn matching_exact_values() {
        assert_read_ok(table_negative(), 409, Some(40));
        assert_read_ok(table_negative(), 819, Some(30));
        assert_read_ok(table_negative(), 1228, Some(10));
    }

    #[test]
    fn interpolates() {
        assert_read_ok(table_negative(), 502, Some(38));
        assert_read_ok(table_negative(), 614, Some(35));
        assert_read_ok(table_negative(), 1023, Some(21));
    }

    #[test]
    fn outside_range() {
        assert_read_ok(table_negative(), 0, None);
        assert_read_ok(table_negative(), 408, None);
        assert_read_ok(table_negative(), 1229, None);
        assert_read_ok(table_negative(), 10000, None);
    }

    #[test]
    fn error() {
        let mut adc =
            adc(&[Transaction::read(0, 0).with_error(MockError::Io(ErrorKind::InvalidData))]);
        assert!(interpolator(table_positive()).read(&mut adc).is_err());
    }

    #[test]
    fn min_value() {
        assert_eq!(interpolator(table_positive()).min_value(), 10);
        assert_eq!(interpolator(table_negative()).min_value(), 10);
    }

    #[test]
    fn max_value() {
        assert_eq!(interpolator(table_positive()).max_value(), 40);
        assert_eq!(interpolator(table_negative()).max_value(), 40);
    }
}
