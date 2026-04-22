//! Single-venue CEX WebSocket stream (same idea as `arb-listener`).

use aeon_market_scanner_rs::{CEXTrait, CexPrice, MarketScannerError};
use tokio::sync::mpsc;

/// Connects to a single exchange via WebSocket and forwards price updates
/// to the shared `price_tx` channel.
///
/// Opens the stream, then loops indefinitely — forwarding each incoming
/// price update until the WebSocket closes or the receiver is dropped.
///
/// # Arguments
/// * `exchange`           - The exchange to connect to; must implement `CEXTrait`
/// * `symbols`            - List of trading pairs to subscribe e.g. `["BTC-USDT", "ETH-USDT"]`
/// * `reconnect_attempts` - How many times to retry on connection failure
/// * `reconnect_delay_ms` - Delay in milliseconds between reconnect attempts
/// * `price_tx`           - Shared sender channel; all exchanges write to the same receiver
///
/// # Errors
/// Returns `Err` if the WebSocket connection fails to establish.
pub async fn run_cex_listener<E>(
    exchange: E,
    symbols: Vec<String>,
    reconnect_attempts: u32,
    reconnect_delay_ms: u64,
    price_tx: mpsc::Sender<CexPrice>,
) -> Result<(), MarketScannerError>
where
    E: CEXTrait + Send + Sync,
{
    let symbols_str: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
    let mut rx = exchange
        .stream_price_websocket(&symbols_str, reconnect_attempts, reconnect_delay_ms)
        .await?;

    while let Some(update) = rx.recv().await {
        let _ = price_tx.send(update).await;
    }

    Ok(())
}
