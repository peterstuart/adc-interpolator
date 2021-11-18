#![cfg_attr(not(test), no_std)]

//! An interpolator for analog-to-digital converters.
//!
//! Convert voltage readings from an ADC into meaningful values using
//! linear interpolation.
//!
//! # Examples
//!
//! ```
//! use adc_interpolator::{AdcInterpolator, pair};
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
//! let mut interpolator = AdcInterpolator::new(
//!     pin,
//!     [
//!         // 1000 mV maximum voltage, 12-bit precision
//!         pair(1000, 12, 100, 40), // 100 mV -> 40
//!         pair(1000, 12, 200, 30), // 200 mV -> 30
//!         pair(1000, 12, 300, 10), // 300 mV -> 10
//!     ],
//! );
//!
//! // With voltage at 150 mV, the value is 35
//! assert_eq!(interpolator.read(&mut adc), Ok(Some(35)));
//! ```

mod adc_interpolator;
mod interpolate;

pub use self::adc_interpolator::{pair, AdcInterpolator};
