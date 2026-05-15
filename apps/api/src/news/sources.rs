//! Static list of RSS feed sources.

pub struct Source {
    pub name: &'static str,
    pub url: &'static str,
}

pub const SOURCES: &[Source] = &[
    Source {
        name: "Decrypt",
        url: "https://decrypt.co/feed",
    },
    Source {
        name: "CoinTelegraph",
        url: "https://cointelegraph.com/rss",
    },
    Source {
        name: "NewsBTC",
        url: "https://www.newsbtc.com/feed/",
    },
    Source {
        name: "CoinDesk",
        url: "https://www.coindesk.com/arc/outboundfeeds/rss/",
    },
];
