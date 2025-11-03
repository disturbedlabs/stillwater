use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Check if current tick is within position's range
pub fn is_in_range(current_tick: i32, tick_lower: i32, tick_upper: i32) -> bool {
    current_tick >= tick_lower && current_tick < tick_upper
}

/// Calculate distance to the nearest range edge
pub fn distance_to_range_edge(current_tick: i32, tick_lower: i32, tick_upper: i32) -> i32 {
    if current_tick < tick_lower {
        return 0; // Out of range below
    }
    if current_tick >= tick_upper {
        return 0; // Out of range above
    }

    let dist_to_lower = current_tick - tick_lower;
    let dist_to_upper = tick_upper - current_tick;

    dist_to_lower.min(dist_to_upper)
}

/// Convert tick to price using Uniswap v3/v4 formula: price = 1.0001^tick
pub fn tick_to_price(tick: i32) -> Decimal {
    // For simplicity, using approximation: price ≈ 1.0001^tick
    // In production, would use exact calculation with fixed-point math

    let base = Decimal::from_str("1.0001").unwrap();

    if tick >= 0 {
        base.powi(tick as i64)
    } else {
        Decimal::ONE / base.powi((-tick) as i64)
    }
}

/// Convert price to tick (inverse of tick_to_price)
pub fn price_to_tick(price: Decimal) -> i32 {
    if price <= Decimal::ZERO {
        return 0;
    }

    // tick = log(price) / log(1.0001)
    // Using approximation for now
    let log_price = price.ln();
    let log_base = Decimal::from_str("1.0001").unwrap().ln();

    (log_price / log_base).round().to_i32().unwrap_or(0)
}

/// Calculate range width as a percentage
pub fn range_width_percent(tick_lower: i32, tick_upper: i32) -> Decimal {
    let price_lower = tick_to_price(tick_lower);
    let price_upper = tick_to_price(tick_upper);

    if price_lower.is_zero() {
        return Decimal::ZERO;
    }

    ((price_upper - price_lower) / price_lower) * Decimal::from(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_range() {
        assert!(is_in_range(100, 50, 150));
        assert!(is_in_range(50, 50, 150));
        assert!(!is_in_range(150, 50, 150));
        assert!(!is_in_range(30, 50, 150));
        assert!(!is_in_range(200, 50, 150));
    }

    #[test]
    fn test_distance_to_range_edge() {
        assert_eq!(distance_to_range_edge(100, 50, 150), 50);
        assert_eq!(distance_to_range_edge(75, 50, 150), 25);
        assert_eq!(distance_to_range_edge(125, 50, 150), 25);
        assert_eq!(distance_to_range_edge(30, 50, 150), 0);
        assert_eq!(distance_to_range_edge(200, 50, 150), 0);
    }

    #[test]
    fn test_tick_to_price() {
        let price_0 = tick_to_price(0);
        assert!((price_0 - Decimal::ONE).abs() < Decimal::from_str("0.0001").unwrap());

        // Positive tick should increase price
        let price_100 = tick_to_price(100);
        assert!(price_100 > Decimal::ONE);

        // Negative tick should decrease price
        let price_neg100 = tick_to_price(-100);
        assert!(price_neg100 < Decimal::ONE);
    }
}
