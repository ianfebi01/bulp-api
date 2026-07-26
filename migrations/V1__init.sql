CREATE TABLE IF NOT EXISTS bulb_state (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    is_on       BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO bulb_state (id, is_on)
VALUES (1, FALSE)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS schedules (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    cron_expr  TEXT NOT NULL,
    action     TEXT NOT NULL CHECK (action IN ('on', 'off')),
    enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);