# adc-interpolator &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs Badge]][docs] 
[Build Status]: https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fpeterstuart%2Fgp2d12%2Fbadge%3Fref%3Dmain&style=flat
[actions]: https://actions-badge.atrox.dev/peterstuart/gp2d12/goto?ref=main
[Latest Version]: https://img.shields.io/crates/v/adc-interpolator.svg
[crates.io]: https://crates.io/crates/adc-interpolator
[Docs Badge]: https://docs.rs/adc-interpolator/badge.svg
[docs]: https://docs.rs/adc-interpolator

An interpolator for analog-to-digital converters.

Convert voltage readings from an ADC into meaningful values using
linear interpolation.

## Examples

```rust
use adc_interpolator::{AdcInterpolator, Config};

let config = Config {
    max_voltage: 1000, // 1000 mV maximum voltage
    precision: 12,     // 12-bit precision
    voltage_to_values: [
        (100, 40), // 100 mV -> 40
        (200, 30), // 200 mV -> 30
        (300, 10), // 300 mV -> 10
    ],
};

let mut interpolator = AdcInterpolator::new(pin, config);

// With voltage at 150 mV, the value is 35
assert_eq!(interpolator.read(&mut adc), Ok(Some(35)));
```
