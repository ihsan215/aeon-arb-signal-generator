use aeon_market_scanner_rs::CexPrice;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Minimal quote snapshot used in arbitrage opportunity detection.
#[derive(Debug, Clone)]
pub struct QuoteSnapshot {
    pub bid_price: f64,
    pub ask_price: f64,
}

impl From<&CexPrice> for QuoteSnapshot {
    fn from(value: &CexPrice) -> Self {
        Self {
            bid_price: value.bid_price,
            ask_price: value.ask_price,
        }
    }
}
/// Pair-level strategy target read from `[[exchanges.pairs]]`.
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangePairConfig {
    pub symbol: String,
}

/// Exchange-level strategy settings read from `[[exchanges]]`.
#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeStrategyConfig {
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub pairs: Vec<ExchangePairConfig>,
}

#[derive(Debug, Deserialize)]
pub struct StrategiesConfig {
    pub reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    #[serde(default)]
    pub exchanges: Vec<ExchangeStrategyConfig>,
}

/// Loads arbitrage strategy configuration from a TOML file path.
///
/// This function reads the file from disk, deserializes the TOML,
/// and returns the configured exchange strategies.
pub fn load_strategies_config(
    path: impl AsRef<Path>,
) -> Result<StrategiesConfig, Box<dyn std::error::Error>> {
    let config_text = fs::read_to_string(path)?;
    let parsed: StrategiesConfig = toml::from_str(&config_text)?;
    Ok(parsed)
}

/// Builds `exchange -> symbols` subscriptions from enabled config entries.
///
/// The listener layer expects symbols in `BASE-QUOTE` format, so this function
/// normalizes values like `ETH/USDT` into `ETH-USDT`.
pub fn exchange_symbols_from_config(cfg: &StrategiesConfig) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();

    for exchange in &cfg.exchanges {
        if !exchange.enabled {
            continue;
        }

        let symbols: Vec<String> = exchange
            .pairs
            .iter()
            .map(|pair| pair.symbol.replace('/', "-"))
            .collect();

        if !symbols.is_empty() {
            out.insert(exchange.name.to_lowercase(), symbols);
        }
    }

    out
}
