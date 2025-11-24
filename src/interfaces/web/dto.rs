use crate::domain::painting::value_objects::DrawingStrategy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyStats {
    pub strategy: DrawingStrategy,
    pub dpad_operations: usize,
    pub a_button_presses: usize,
    pub estimated_time_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyComparisonResponse {
    pub strategies: Vec<StrategyStats>,
}
