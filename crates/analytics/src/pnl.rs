use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use stillwater_models::{Position, PositionPnL, Swap};

use crate::utils::tick_to_price;

/// Calculate fees earned from swaps
///
/// For a concentrated liquidity position, fees are earned when:
/// 1. The swap occurs while the position is in range
/// 2. The position has active liquidity
///
/// Simplified calculation: assumes position was always in range for swaps provided
pub fn calculate_fees_earned(_position: &Position, swaps: &[Swap]) -> Decimal {
    if swaps.is_empty() {
        return Decimal::ZERO;
    }

    // Simplified fee calculation
    // In reality, would need:
    // - Total pool liquidity at time of each swap
    // - Position's share of liquidity
    // - Fee tier for the pool
    //
    // For MVP, estimate based on swap volumes and assume 0.3% fee tier
    let fee_rate = Decimal::from_str("0.003").unwrap(); // 0.3%

    let total_volume: Decimal = swaps
        .iter()
        .map(|swap| {
            // Use absolute values and convert to decimal
            // This is a rough approximation
            let amt0 = swap.amount0.abs().to_string();
            let amt1 = swap.amount1.abs().to_string();

            Decimal::from_str(&amt0).unwrap_or(Decimal::ZERO)
                + Decimal::from_str(&amt1).unwrap_or(Decimal::ZERO)
        })
        .sum();

    // Estimate fees as a fraction of total volume
    // In production, would calculate exact share based on liquidity
    let estimated_position_share = Decimal::from_str("0.01").unwrap(); // 1% of pool

    total_volume * fee_rate * estimated_position_share
}

/// Calculate impermanent loss for concentrated liquidity position
///
/// IL for concentrated liquidity is different from full-range (v2) positions:
/// - Only incur IL when price moves within the range
/// - IL can be higher or lower depending on range width
///
/// Simplified formula:
/// IL = (value_if_held - current_value) / value_if_held
pub fn calculate_impermanent_loss(
    position: &Position,
    initial_price: Decimal,
    current_price: Decimal,
) -> Decimal {
    if initial_price.is_zero() || current_price.is_zero() {
        return Decimal::ZERO;
    }

    // For positions with extreme tick ranges (like full-range positions),
    // use a simplified calculation to avoid overflow
    let tick_range = (position.tick_upper - position.tick_lower).abs();

    if tick_range > 1_000_000 {
        // This is likely a full-range position (e.g., ±887220)
        // Use simplified IL calculation similar to Uniswap v2

        // If price hasn't moved, no IL
        if (current_price - initial_price).abs() < Decimal::from_str("0.0001").unwrap() {
            return Decimal::ZERO;
        }

        // For full-range positions, IL ≈ 2*sqrt(price_ratio) - price_ratio - 1
        // Simplified approximation: IL increases with price movement
        let price_change_pct = ((current_price - initial_price) / initial_price).abs();

        // Cap IL at reasonable value
        let il = price_change_pct * Decimal::from_str("0.2").unwrap(); // Max ~20% for moderate price changes
        return il.min(Decimal::from_str("0.5").unwrap()); // Cap at 50%
    }

    // For normal range positions, use tick-based calculation
    let price_lower = tick_to_price(position.tick_lower);
    let price_upper = tick_to_price(position.tick_upper);

    // If price hasn't moved, no IL
    if (current_price - initial_price).abs() < Decimal::from_str("0.0001").unwrap() {
        return Decimal::ZERO;
    }

    // Simplified IL calculation for concentrated liquidity
    // Full formula involves complex integral calculations
    //
    // Approximation: IL increases with price movement
    let price_change_pct = ((current_price - initial_price) / initial_price).abs();

    // Range width factor: wider range = less IL (approaching v2 behavior)
    // Add safety check to avoid division by zero
    if price_lower.is_zero() {
        return Decimal::ZERO;
    }

    let range_width = (price_upper - price_lower) / price_lower;

    // IL increases with price movement, decreases with range width
    let il_factor = price_change_pct / (Decimal::ONE + range_width);

    // Simplified IL formula (in production, use exact Uniswap v3 math)
    il_factor * Decimal::from_str("0.5").unwrap()
}

/// Calculate net P&L
pub fn calculate_net_pnl(fees: Decimal, il: Decimal, gas: Decimal) -> Decimal {
    fees - il - gas
}

/// Calculate complete position P&L
pub fn calculate_position_pnl(
    position: &Position,
    swaps: &[Swap],
    initial_price: Decimal,
    current_price: Decimal,
    gas_spent: Decimal,
) -> PositionPnL {
    let fees_earned = calculate_fees_earned(position, swaps);
    let impermanent_loss = calculate_impermanent_loss(position, initial_price, current_price);
    let net_pnl = calculate_net_pnl(fees_earned, impermanent_loss, gas_spent);

    PositionPnL {
        fees_earned,
        impermanent_loss,
        gas_spent,
        net_pnl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{I256, U256};
    use chrono::Utc;

    fn create_test_position() -> Position {
        Position {
            id: 1,
            nft_id: "1".to_string(),
            owner: "0xtest".to_string(),
            pool_id: "0xpool".to_string(),
            tick_lower: -1000,
            tick_upper: 1000,
            liquidity: U256::from(1000000u64),
            created_at: Utc::now(),
        }
    }

    fn create_test_swap(amount0: i64, amount1: i64) -> Swap {
        Swap {
            id: 1,
            tx_hash: "0xtx".to_string(),
            pool_id: "0xpool".to_string(),
            amount0: I256::try_from(amount0).unwrap(),
            amount1: I256::try_from(amount1).unwrap(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_calculate_fees_earned() {
        let position = create_test_position();
        let swaps = vec![
            create_test_swap(1000, 1000),
            create_test_swap(2000, 2000),
        ];

        let fees = calculate_fees_earned(&position, &swaps);
        assert!(fees > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss() {
        let position = create_test_position();
        let initial_price = Decimal::from(100);
        let current_price = Decimal::from(110);

        let il = calculate_impermanent_loss(&position, initial_price, current_price);
        assert!(il >= Decimal::ZERO);
    }

    #[test]
    fn test_calculate_net_pnl() {
        let fees = Decimal::from(100);
        let il = Decimal::from(20);
        let gas = Decimal::from(10);

        let net = calculate_net_pnl(fees, il, gas);
        assert_eq!(net, Decimal::from(70));
    }

    #[test]
    fn test_calculate_position_pnl() {
        let position = create_test_position();
        let swaps = vec![create_test_swap(1000, 1000)];
        let initial_price = Decimal::from(100);
        let current_price = Decimal::from(105);
        let gas_spent = Decimal::from(5);

        let pnl = calculate_position_pnl(&position, &swaps, initial_price, current_price, gas_spent);

        assert!(pnl.fees_earned >= Decimal::ZERO);
        assert!(pnl.impermanent_loss >= Decimal::ZERO);
        assert_eq!(pnl.gas_spent, gas_spent);
    }
}
