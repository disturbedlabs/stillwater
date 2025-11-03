/// GraphQL query to fetch positions by owner address
pub const POSITIONS_BY_OWNER: &str = r#"
query PositionsByOwner($owner: String!) {
  positions(where: { owner: $owner }) {
    id
    owner
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      fee
      tickSpacing
    }
    tickLower
    tickUpper
    liquidity
    transaction {
      timestamp
    }
  }
}
"#;

/// GraphQL query to fetch positions by pool ID
pub const POSITIONS_BY_POOL: &str = r#"
query PositionsByPool($poolId: String!) {
  positions(where: { pool: $poolId }) {
    id
    owner
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      fee
      tickSpacing
    }
    tickLower
    tickUpper
    liquidity
    transaction {
      timestamp
    }
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

/// GraphQL query to fetch all recent positions (for polling)
pub const RECENT_POSITIONS: &str = r#"
query RecentPositions($timestamp: BigInt!) {
  positions(
    where: { transaction_: { timestamp_gte: $timestamp } }
    orderBy: transaction__timestamp
    orderDirection: desc
    first: 100
  ) {
    id
    owner
    pool {
      id
      token0 {
        id
      }
      token1 {
        id
      }
      fee
      tickSpacing
    }
    tickLower
    tickUpper
    liquidity
    transaction {
      timestamp
    }
  }
}
"#;
