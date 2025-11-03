use serde::{Deserialize, Serialize};

/// GraphQL response wrapper
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

/// Response data for positions query
#[derive(Debug, Deserialize)]
pub struct PositionsData {
    pub positions: Vec<PositionResponse>,
}

/// Response data for swaps query
#[derive(Debug, Deserialize)]
pub struct SwapsData {
    pub swaps: Vec<SwapResponse>,
}

/// Position from The Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResponse {
    pub id: String,
    pub owner: String,
    pub pool: PoolResponse,
    #[serde(rename = "tickLower")]
    pub tick_lower: String,
    #[serde(rename = "tickUpper")]
    pub tick_upper: String,
    pub liquidity: String,
    pub transaction: TransactionResponse,
}

/// Pool information from The Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolResponse {
    pub id: String,
    pub token0: TokenResponse,
    pub token1: TokenResponse,
    pub fee: String,
    #[serde(rename = "tickSpacing")]
    pub tick_spacing: String,
}

/// Token information from The Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub id: String,
}

/// Transaction information from The Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    #[serde(default)]
    pub id: Option<String>,
    pub timestamp: String,
}

/// Swap from The Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResponse {
    pub id: String,
    pub transaction: TransactionResponse,
    pub pool: PoolIdResponse,
    pub amount0: String,
    pub amount1: String,
}

/// Simple pool ID response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolIdResponse {
    pub id: String,
}
