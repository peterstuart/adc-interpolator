# adc-interpolator

An interpolator for analog-to-digital converters.

Convert voltage readings from an ADC into meaningful values using
linear interpolation.

## Examples

```rust
use adc_interpolator::{AdcInterpolator, pair};

let mut interpolator = AdcInterpolator::new(
    pin,
    [
        // 1000 mV maximum voltage, 12-bit precision
        pair(1000, 12, 100, 40), // 100 mV -> 40
        pair(1000, 12, 200, 30), // 200 mV -> 30
        pair(1000, 12, 300, 10), // 300 mV -> 10
    ],
);

// With voltage at 150 mV, the value is 35
assert_eq!(interpolator.read(&mut adc), Ok(Some(35)));
```

License: MIT
