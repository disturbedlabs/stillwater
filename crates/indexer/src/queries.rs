/// GraphQL query to fetch modify liquidity events by origin (owner)
pub const POSITIONS_BY_OWNER: &str = r#"
query ModifyLiquidityByOrigin($owner: String!) {
  modifyLiquidities(
    where: { origin: $owner, amount_gt: "0" }
    orderBy: timestamp
    orderDirection: desc
    first: 100
  ) {
    id
    timestamp
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      feeTier
      tickSpacing
    }
    tickLower
    tickUpper
    amount
    origin
  }
}
"#;

/// GraphQL query to fetch modify liquidity events by pool ID
pub const POSITIONS_BY_POOL: &str = r#"
query ModifyLiquidityByPool($poolId: String!) {
  modifyLiquidities(
    where: { pool: $poolId, amount_gt: "0" }
    orderBy: timestamp
    orderDirection: desc
    first: 100
  ) {
    id
    timestamp
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      feeTier
      tickSpacing
    }
    tickLower
    tickUpper
    amount
    origin
  }
}
"#;

/// GraphQL query to fetch recent swaps for a pool
pub const RECENT_SWAPS: &str = r#"
query RecentSwaps($poolId: String!, $timestamp: BigInt!) {
  swaps(
    where: { pool: $poolId, timestamp_gte: $timestamp }
    orderBy: timestamp
    orderDirection: asc
  ) {
    id
    transaction {
      id
      timestamp
    }
    pool {
      id
    }
    amount0
    amount1
  }
}
"#;

/// GraphQL query to fetch all recent modify liquidity events (for polling)
pub const RECENT_POSITIONS: &str = r#"
query RecentModifyLiquidity($timestamp: BigInt!) {
  modifyLiquidities(
    where: { timestamp_gte: $timestamp, amount_gt: "0" }
    orderBy: timestamp
    orderDirection: desc
    first: 100
  ) {
    id
    timestamp
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      feeTier
      tickSpacing
    }
    tickLower
    tickUpper
    amount
    origin
  }
}
"#;
