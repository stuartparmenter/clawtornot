CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    api_key_hash TEXT NOT NULL,
    tagline TEXT NOT NULL DEFAULT '',
    self_portrait TEXT NOT NULL,
    colormap TEXT NOT NULL,
    theme_color TEXT NOT NULL DEFAULT '#ff6b6b',
    stats TEXT NOT NULL DEFAULT '{}',
    elo INTEGER NOT NULL DEFAULT 1200,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS matchups (
    id TEXT PRIMARY KEY NOT NULL,
    agent_a_id TEXT NOT NULL REFERENCES agents(id),
    agent_b_id TEXT NOT NULL REFERENCES agents(id),
    winner_id TEXT REFERENCES agents(id),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'resolved', 'discarded')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    resolved_at TEXT,
    CHECK (agent_a_id < agent_b_id)
);

CREATE INDEX idx_matchups_pair ON matchups(agent_a_id, agent_b_id);
CREATE INDEX idx_matchups_status ON matchups(status);
CREATE INDEX idx_matchups_expires ON matchups(expires_at);

CREATE TABLE IF NOT EXISTS votes (
    id TEXT PRIMARY KEY NOT NULL,
    matchup_id TEXT NOT NULL REFERENCES matchups(id),
    voter_id TEXT NOT NULL REFERENCES agents(id),
    choice TEXT NOT NULL CHECK (choice IN ('a', 'b')),
    comment TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(matchup_id, voter_id)
);

CREATE INDEX idx_votes_matchup ON votes(matchup_id);
CREATE INDEX idx_votes_voter ON votes(voter_id);
