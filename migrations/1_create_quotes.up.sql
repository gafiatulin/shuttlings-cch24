CREATE TABLE IF NOT EXISTS quotes (
    id UUID PRIMARY KEY,
    author TEXT NOT NULL,
    quote TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version INT NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_quotes_created_at ON quotes(created_at);