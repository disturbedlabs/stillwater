use rust_decimal::Decimal;
use stillwater_models::{HealthStatus, Position, PositionPnL};

use crate::utils::{distance_to_range_edge, is_in_range};

/// Determine position health status based on current tick and P&L
///
/// Rules:
/// - Healthy: in range + positive P&L
/// - Warning: within 10% of range edge
/// - Critical: out of range OR negative P&L
pub fn get_position_health(
    position: &Position,
    current_tick: i32,
    pnl: &PositionPnL,
) -> HealthStatus {
    // Critical if out of range
    if !is_in_range(current_tick, position.tick_lower, position.tick_upper) {
        return HealthStatus::Critical;
    }

    // Critical if negative P&L
    if pnl.net_pnl < Decimal::ZERO {
        return HealthStatus::Critical;
    }

    // Warning if within 10% of range edge
    let distance = distance_to_range_edge(current_tick, position.tick_lower, position.tick_upper);
    let range_width = position.tick_upper - position.tick_lower;

    if distance < range_width / 10 {
        return HealthStatus::Warning;
    }

    // Otherwise healthy
    HealthStatus::Healthy
}

/// Get detailed health information as a string
pub fn get_health_details(
    position: &Position,
    current_tick: i32,
    pnl: &PositionPnL,
) -> String {
    let status = get_position_health(position, current_tick, pnl);
    let in_range = is_in_range(current_tick, position.tick_lower, position.tick_upper);
    let distance = distance_to_range_edge(current_tick, position.tick_lower, position.tick_upper);

    format!(
        "Status: {:?}, In Range: {}, Distance to Edge: {}, Net P&L: {}",
        status, in_range, distance, pnl.net_pnl
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;
    use chrono::Utc;

    fn create_test_position(tick_lower: i32, tick_upper: i32) -> Position {
        Position {
            id: 1,
            nft_id: "1".to_string(),
            owner: "0xtest".to_string(),
            pool_id: "0xpool".to_string(),
            tick_lower,
            tick_upper,
            liquidity: U256::from(1000000u64),
            created_at: Utc::now(),
        }
    }

    fn create_test_pnl(net_pnl: i64) -> PositionPnL {
        PositionPnL {
            fees_earned: Decimal::from(100),
            impermanent_loss: Decimal::from(20),
            gas_spent: Decimal::from(10),
            net_pnl: Decimal::from(net_pnl),
        }
    }

    #[test]
    fn test_healthy_status() {
        let position = create_test_position(-1000, 1000);
        let pnl = create_test_pnl(70); // Positive P&L
        let current_tick = 0; // Center of range

        let health = get_position_health(&position, current_tick, &pnl);
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[test]
    fn test_warning_status_near_edge() {
        let position = create_test_position(-1000, 1000);
        let pnl = create_test_pnl(70); // Positive P&L
        let current_tick = 950; // Within 10% of upper edge

        let health = get_position_health(&position, current_tick, &pnl);
        assert_eq!(health, HealthStatus::Warning);
    }

    #[test]
    fn test_critical_status_out_of_range() {
        let position = create_test_position(-1000, 1000);
        let pnl = create_test_pnl(70); // Positive P&L but out of range
        let current_tick = 1500; // Out of range

        let health = get_position_health(&position, current_tick, &pnl);
        assert_eq!(health, HealthStatus::Critical);
    }

    #[test]
    fn test_critical_status_negative_pnl() {
        let position = create_test_position(-1000, 1000);
        let pnl = create_test_pnl(-10); // Negative P&L
        let current_tick = 0; // In range but negative P&L

        let health = get_position_health(&position, current_tick, &pnl);
        assert_eq!(health, HealthStatus::Critical);
    }

    #[test]
    fn test_get_health_details() {
        let position = create_test_position(-1000, 1000);
        let pnl = create_test_pnl(70);
        let current_tick = 0;

        let details = get_health_details(&position, current_tick, &pnl);
        assert!(details.contains("Healthy"));
        assert!(details.contains("In Range: true"));
    }
}
