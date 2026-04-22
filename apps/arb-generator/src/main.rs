mod config;
mod cex_listener;

use crate::config::{exchange_symbols_from_config, load_strategies_config};
use aeon_market_scanner_rs::CexPrice;
use cex_listener::spawn_configured_cex_listeners;

#[tokio::main]
async fn main() {
    // --- Config (stderr + exit 1 on failure, no panic) ---
    let cfg = match load_strategies_config("arb_config.toml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let merged = exchange_symbols_from_config(&cfg);
    if merged.is_empty() {
        eprintln!("nothing to subscribe — enable at least one exchange pair in arb_config.toml");
        std::process::exit(1);
    }

    // Bounded channel: listener tasks `send`, consumer task `recv`.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<CexPrice>(2048);

    // Spawn one async listener per enabled exchange.
    let handles = match spawn_configured_cex_listeners(
        merged,
        cfg.reconnect_attempts,
        cfg.reconnect_delay_ms,
        tx.clone(),
    ) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    drop(tx);

    let streams_join = tokio::spawn(async move {
        for h in handles {
            match h.await {
                Ok(Err(e)) => eprintln!("CEX listener error: {e}"),
                Err(e) => eprintln!("CEX listener panicked: {e:?}"),
                Ok(Ok(())) => {}
            }
        }
    });

    let on_update = tokio::spawn(async move {
        while let Some(p) = rx.recv().await {
            println!(
                "[price] exchange={:?} symbol={} bid={} ask={}",
                p.exchange, p.symbol, p.bid_price, p.ask_price
            );
        }
    });

    // Shutdown on Ctrl+C, when all listeners finish, or when consumer ends.
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nShutting down...");
        }
        r = streams_join => {
            if let Err(e) = r {
                eprintln!("streams join task panicked: {e:?}");
            }
        }
        r = on_update => {
            if let Err(e) = r {
                eprintln!("update task panicked: {e:?}");
            }
        }
    }
}
