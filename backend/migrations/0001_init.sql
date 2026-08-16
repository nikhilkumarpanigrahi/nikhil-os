-- NIKHIL//OS backend — initial schema.
-- Messages from the Contact app, plus an audit trail for admin actions.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE messages (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    email       text NOT NULL CHECK (char_length(email) <= 254),
    subject     text NOT NULL DEFAULT '' CHECK (char_length(subject) <= 120),
    body        text NOT NULL CHECK (char_length(body) BETWEEN 1 AND 4000),
    topic       text NOT NULL DEFAULT 'general',
    origin      text,
    user_agent  text,
    ip          inet,
    status      text NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'read', 'replied', 'archived')),
    created_at  timestamptz NOT NULL DEFAULT now(),
    read_at     timestamptz
);

CREATE INDEX messages_status_idx ON messages (status, created_at DESC);
CREATE INDEX messages_created_idx ON messages (created_at DESC);

CREATE TABLE admin_events (
    id         bigserial PRIMARY KEY,
    kind       text NOT NULL,
    detail     jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX admin_events_created_idx ON admin_events (created_at DESC);
