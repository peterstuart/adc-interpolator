pub fn interpolate(x0: u32, x1: u32, y0: u32, y1: u32, x: u32) -> u32 {
    if y0 > y1 {
        y0 - (x - x0) * (y0 - y1) / (x1 - x0)
    } else {
        y0 + (x - x0) * (y1 - y0) / (x1 - x0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn interpolate() {
        assert_eq!(super::interpolate(0, 10, 0, 100, 0), 0);
        assert_eq!(super::interpolate(0, 10, 0, 100, 2), 20);
        assert_eq!(super::interpolate(0, 10, 0, 100, 5), 50);
        assert_eq!(super::interpolate(0, 10, 0, 100, 8), 80);
        assert_eq!(super::interpolate(0, 10, 0, 100, 10), 100);
    }

    #[test]
    fn interpolate_flipped_y() {
        assert_eq!(super::interpolate(0, 10, 100, 0, 0), 100);
        assert_eq!(super::interpolate(0, 10, 100, 0, 2), 80);
        assert_eq!(super::interpolate(0, 10, 100, 0, 5), 50);
        assert_eq!(super::interpolate(0, 10, 100, 0, 8), 20);
        assert_eq!(super::interpolate(0, 10, 100, 0, 10), 0);
    }
}
