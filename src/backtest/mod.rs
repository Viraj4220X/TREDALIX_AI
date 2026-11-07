mod config;
mod engine;
mod execution;
mod metrics;
mod portfolio;
mod report;
mod risk;
mod strategy;

pub use config::{BacktestConfig, EndOfDataPolicy};
pub use engine::{BacktestResult, run_backtest};
pub use metrics::{BenchmarkMetrics, PerformanceMetrics};
pub use portfolio::{EquityPoint, OrderRecord, OrderStatus, Side, TradeRecord};
pub use report::write_report;
pub use strategy::{Action, SmaRsiStrategy, Strategy};
