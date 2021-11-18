# adc-interpolator

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

License: MIT
