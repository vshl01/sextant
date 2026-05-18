CREATE TABLE coins (
    symbol           TEXT             PRIMARY KEY,
    coingecko_id     TEXT             NOT NULL,
    binance_pair     TEXT             NOT NULL,
    name             TEXT             NOT NULL,
    icon_url         TEXT,
    market_cap_usd   DOUBLE PRECISION,
    circulating      DOUBLE PRECISION,
    change_1h_pct    DOUBLE PRECISION,
    change_24h_pct   DOUBLE PRECISION,
    change_7d_pct    DOUBLE PRECISION,
    refreshed_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);
