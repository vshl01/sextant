//! Static list of coins we track. Maps a unified ticker to the upstream
//! identifiers each external API uses for the same coin.

pub struct Source {
    pub symbol: &'static str,       // 'BTC' — uppercase, our canonical id
    pub coingecko_id: &'static str, // 'bitcoin' — what CoinGecko calls it
    pub binance_pair: &'static str, // 'BTCUSDT' — what Binance trades it as
    pub name: &'static str,         // 'Bitcoin' — display name
}

pub const COINS: &[Source] = &[
    Source {
        symbol: "BTC",
        coingecko_id: "bitcoin",
        binance_pair: "BTCUSDT",
        name: "Bitcoin",
    },
    Source {
        symbol: "ETH",
        coingecko_id: "ethereum",
        binance_pair: "ETHUSDT",
        name: "Ethereum",
    },
    Source {
        symbol: "SOL",
        coingecko_id: "solana",
        binance_pair: "SOLUSDT",
        name: "Solana",
    },
    Source {
        symbol: "XRP",
        coingecko_id: "ripple",
        binance_pair: "XRPUSDT",
        name: "XRP",
    },
    Source {
        symbol: "DOGE",
        coingecko_id: "dogecoin",
        binance_pair: "DOGEUSDT",
        name: "Dogecoin",
    },
    Source {
        symbol: "ADA",
        coingecko_id: "cardano",
        binance_pair: "ADAUSDT",
        name: "Cardano",
    },
    Source {
        symbol: "AVAX",
        coingecko_id: "avalanche-2",
        binance_pair: "AVAXUSDT",
        name: "Avalanche",
    },
    Source {
        symbol: "LINK",
        coingecko_id: "chainlink",
        binance_pair: "LINKUSDT",
        name: "Chainlink",
    },
    Source {
        symbol: "MATIC",
        coingecko_id: "matic-network",
        binance_pair: "MATICUSDT",
        name: "Polygon",
    },
    Source {
        symbol: "BNB",
        coingecko_id: "binancecoin",
        binance_pair: "BNBUSDT",
        name: "BNB",
    },
];
