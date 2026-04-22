use aeon_market_scanner_rs::CexPrice;
use aeon_market_scanner_rs::{
    Binance, Bitfinex, Bitget, Bybit, Coinbase, Cryptocom, Gateio, Kraken, Kucoin,
    MarketScannerError, Mexc, Upbit, OKX,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::stream;

/// Spawns one async listener task per exchange.
/// Each task opens a WebSocket connection to its exchange and forwards
/// incoming price updates to the shared `tx` channel.
///
/// # Arguments
/// * `exchange_symbols` - Map of exchange ID to its subscribed symbols e.g. `"binance" → ["BTC-USDT"]`
/// * `reconnect_attempts` - How many times to retry on connection failure
/// * `reconnect_delay_ms` - Delay in milliseconds between reconnect attempts
/// * `tx` - Shared sender channel; all tasks write price updates to the same receiver
///
/// # Errors
/// Returns `Err` if an unrecognized exchange ID is found in `exchange_symbols`.
pub fn spawn_configured_cex_listeners(
    exchange_symbols: HashMap<String, Vec<String>>,
    reconnect_attempts: u32,
    reconnect_delay_ms: u64,
    tx: mpsc::Sender<CexPrice>,
) -> Result<Vec<JoinHandle<Result<(), MarketScannerError>>>, String> {
    let mut handles = Vec::new();

    for (ex_id, symbols_vec) in exchange_symbols {
        let tx = tx.clone();
        let h = match ex_id.as_str() {
            "binance" => tokio::spawn(stream::run_cex_listener(
                Binance::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "okx" => tokio::spawn(stream::run_cex_listener(
                OKX::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "bybit" => tokio::spawn(stream::run_cex_listener(
                Bybit::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "gateio" => tokio::spawn(stream::run_cex_listener(
                Gateio::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "kucoin" => tokio::spawn(stream::run_cex_listener(
                Kucoin::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "bitget" => tokio::spawn(stream::run_cex_listener(
                Bitget::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "kraken" => tokio::spawn(stream::run_cex_listener(
                Kraken::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "coinbase" => tokio::spawn(stream::run_cex_listener(
                Coinbase::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "bitfinex" => tokio::spawn(stream::run_cex_listener(
                Bitfinex::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "mexc" => tokio::spawn(stream::run_cex_listener(
                Mexc::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "cryptocom" => tokio::spawn(stream::run_cex_listener(
                Cryptocom::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            "upbit" => tokio::spawn(stream::run_cex_listener(
                Upbit::new(),
                symbols_vec,
                reconnect_attempts,
                reconnect_delay_ms,
                tx,
            )),
            _ => {
                return Err(format!(
                    "unknown CEX in config: {:?} (supported: binance, bitfinex, bitget, bybit, coinbase, cryptocom, gateio, kraken, kucoin, mexc, okx, upbit)",
                    ex_id
                ));
            }
        };
        handles.push(h);
    }

    Ok(handles)
}
