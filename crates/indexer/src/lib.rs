mod queries;
mod types;

use alloy::primitives::{I256, U256};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use stillwater_db::{insert_pool, insert_position, insert_swap};
use stillwater_models::{Pool, Position, Swap};
use tracing::{debug, info, warn};

pub use types::*;

/// The Graph indexer client
pub struct GraphIndexer {
    client: Client,
    graph_url: String,
}

impl GraphIndexer {
    /// Create a new Graph indexer client
    pub fn new(graph_url: String) -> Self {
        Self {
            client: Client::new(),
            graph_url,
        }
    }

    /// Create indexer from environment variable
    pub fn from_env() -> Result<Self> {
        let graph_url = std::env::var("GRAPH_API_URL")
            .context("GRAPH_API_URL must be set in environment")?;
        Ok(Self::new(graph_url))
    }

    /// Execute a GraphQL query
    async fn query<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let body = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .client
            .post(&self.graph_url)
            .json(&body)
            .send()
            .await
            .context("Failed to send GraphQL request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GraphQL request failed with status {}: {}", status, text));
        }

        let result: GraphQLResponse<T> = response
            .json()
            .await
            .context("Failed to parse GraphQL response")?;

        if let Some(errors) = result.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(anyhow!("GraphQL errors: {}", error_messages.join(", ")));
        }

        result.data.ok_or_else(|| anyhow!("No data in GraphQL response"))
    }

    /// Fetch positions by owner address
    pub async fn fetch_positions_by_owner(&self, owner: &str) -> Result<Vec<PositionResponse>> {
        let variables = json!({ "owner": owner.to_lowercase() });
        let data: PositionsData = self.query(queries::POSITIONS_BY_OWNER, variables).await?;
        Ok(data.positions)
    }

    /// Fetch positions by pool ID
    pub async fn fetch_positions_by_pool(&self, pool_id: &str) -> Result<Vec<PositionResponse>> {
        let variables = json!({ "poolId": pool_id.to_lowercase() });
        let data: PositionsData = self.query(queries::POSITIONS_BY_POOL, variables).await?;
        Ok(data.positions)
    }

    /// Fetch recent swaps for a pool since a timestamp
    pub async fn fetch_recent_swaps(
        &self,
        pool_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<SwapResponse>> {
        let timestamp = since.timestamp();
        let variables = json!({
            "poolId": pool_id.to_lowercase(),
            "timestamp": timestamp.to_string()
        });
        let data: SwapsData = self.query(queries::RECENT_SWAPS, variables).await?;
        Ok(data.swaps)
    }

    /// Fetch recent positions since a timestamp
    pub async fn fetch_recent_positions(&self, since: DateTime<Utc>) -> Result<Vec<PositionResponse>> {
        let timestamp = since.timestamp();
        let variables = json!({ "timestamp": timestamp.to_string() });
        let data: PositionsData = self.query(queries::RECENT_POSITIONS, variables).await?;
        Ok(data.positions)
    }

    /// Sync positions to database
    pub async fn sync_positions(&self, db_pool: &PgPool) -> Result<usize> {
        // Fetch positions from the last hour
        let since = Utc::now() - chrono::Duration::hours(1);
        let positions = self.fetch_recent_positions(since).await?;

        info!("Fetched {} positions from The Graph", positions.len());

        let mut inserted = 0;
        for pos_resp in positions {
            // First, ensure the pool exists
            match self.convert_and_insert_pool(db_pool, &pos_resp.pool).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to insert pool {}: {}", pos_resp.pool.id, e);
                    continue;
                }
            }

            // Then insert the position
            match self.convert_and_insert_position(db_pool, &pos_resp).await {
                Ok(_) => {
                    inserted += 1;
                    debug!("Inserted position {}", pos_resp.id);
                }
                Err(e) => {
                    warn!("Failed to insert position {}: {}", pos_resp.id, e);
                }
            }
        }

        info!("Inserted {} new positions", inserted);
        Ok(inserted)
    }

    /// Sync swaps to database
    pub async fn sync_swaps(&self, db_pool: &PgPool, pool_id: &str) -> Result<usize> {
        let since = Utc::now() - chrono::Duration::hours(1);
        let swaps = self.fetch_recent_swaps(pool_id, since).await?;

        info!("Fetched {} swaps from The Graph for pool {}", swaps.len(), pool_id);

        let mut inserted = 0;
        for swap_resp in swaps {
            match self.convert_and_insert_swap(db_pool, &swap_resp).await {
                Ok(_) => {
                    inserted += 1;
                    debug!("Inserted swap {}", swap_resp.id);
                }
                Err(e) => {
                    warn!("Failed to insert swap {}: {}", swap_resp.id, e);
                }
            }
        }

        info!("Inserted {} new swaps", inserted);
        Ok(inserted)
    }

    /// Convert and insert pool into database
    async fn convert_and_insert_pool(&self, db_pool: &PgPool, pool_resp: &PoolResponse) -> Result<()> {
        let fee_tier = pool_resp.fee.parse::<i32>()
            .context("Failed to parse fee")?;
        let tick_spacing = pool_resp.tick_spacing.parse::<i32>()
            .context("Failed to parse tick spacing")?;

        let pool = Pool {
            pool_id: pool_resp.id.clone(),
            token0: pool_resp.token0.id.clone(),
            token1: pool_resp.token1.id.clone(),
            fee_tier,
            tick_spacing,
            created_at: Utc::now(), // We don't have creation time from subgraph
        };

        insert_pool(db_pool, &pool).await?;
        Ok(())
    }

    /// Convert and insert position into database
    async fn convert_and_insert_position(&self, db_pool: &PgPool, pos_resp: &PositionResponse) -> Result<()> {
        let tick_lower = pos_resp.tick_lower.parse::<i32>()
            .context("Failed to parse tick_lower")?;
        let tick_upper = pos_resp.tick_upper.parse::<i32>()
            .context("Failed to parse tick_upper")?;
        let liquidity = U256::from_str_radix(&pos_resp.liquidity, 10)
            .context("Failed to parse liquidity")?;
        let timestamp = pos_resp.transaction.timestamp.parse::<i64>()
            .context("Failed to parse timestamp")?;
        let created_at = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?;

        let position = Position {
            id: 0, // Will be auto-generated
            nft_id: pos_resp.id.clone(),
            owner: pos_resp.owner.clone(),
            pool_id: pos_resp.pool.id.clone(),
            tick_lower,
            tick_upper,
            liquidity,
            created_at,
        };

        insert_position(db_pool, &position).await?;
        Ok(())
    }

    /// Convert and insert swap into database
    async fn convert_and_insert_swap(&self, db_pool: &PgPool, swap_resp: &SwapResponse) -> Result<()> {
        let amount0 = swap_resp.amount0.parse::<I256>()
            .context("Failed to parse amount0")?;
        let amount1 = swap_resp.amount1.parse::<I256>()
            .context("Failed to parse amount1")?;
        let timestamp = swap_resp.transaction.timestamp.parse::<i64>()
            .context("Failed to parse timestamp")?;
        let swap_time = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?;

        let tx_hash = swap_resp.transaction.id.clone()
            .unwrap_or_else(|| swap_resp.id.clone());

        let swap = Swap {
            id: 0, // Will be auto-generated
            tx_hash,
            pool_id: swap_resp.pool.id.clone(),
            amount0,
            amount1,
            timestamp: swap_time,
        };

        insert_swap(db_pool, &swap).await?;
        Ok(())
    }
}

/// Helper function to create indexer from environment and sync all data
pub async fn sync_all(db_pool: &PgPool) -> Result<()> {
    let indexer = GraphIndexer::from_env()?;

    let positions_count = indexer.sync_positions(db_pool).await?;
    info!("Synced {} positions", positions_count);

    Ok(())
}
