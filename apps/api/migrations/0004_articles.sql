CREATE TABLE articles (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    source      TEXT         NOT NULL,
    title       TEXT         NOT NULL,
    link        TEXT         NOT NULL UNIQUE,
    snippet     TEXT         NOT NULL DEFAULT '',
    published   TIMESTAMPTZ,
    fetched_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    coins       TEXT[]       NOT NULL DEFAULT '{}'
);

CREATE INDEX articles_published_idx ON articles (published DESC);
CREATE INDEX articles_coins_idx     ON articles USING GIN (coins);
