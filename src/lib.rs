#![cfg_attr(not(test), no_std)]

//! An interpolator for analog-to-digital converters.
//!
//! Convert voltage readings from an ADC into meaningful values using
//! linear interpolation.
//!
//! # Examples
//!
//! ```
//! use adc_interpolator::{AdcInterpolator, Config};
//! # use embedded_hal_mock::{
//! #     adc::{Mock, MockChan0, Transaction},
//! #     common::Generic,
//! #     MockError,
//! # };
//! # let pin = MockChan0 {};
//! # let expectations: [Transaction<u16>; 1] = [Transaction::read(0, 614)];
//! # let mut adc = Mock::new(&expectations);
//! # let pin = MockChan0 {};
//!
//! let config = Config {
//!     max_voltage: 1000, // 1000 mV maximum voltage
//!     precision: 12,     // 12-bit precision
//!     voltage_to_values: [
//!         (100, 40), // 100 mV -> 40
//!         (200, 30), // 200 mV -> 30
//!         (300, 10), // 300 mV -> 10
//!     ],
//! };
//!
//! let mut interpolator = AdcInterpolator::new(pin, config);
//!
//! // With voltage at 150 mV, the value is 35
//! assert_eq!(interpolator.read(&mut adc), Ok(Some(35)));
//! ```

mod adc_interpolator;
mod interpolate;

pub use self::adc_interpolator::{AdcInterpolator, Config};
