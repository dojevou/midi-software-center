-- Migration: 025_projects.sql
-- Description: Project management schema for saving/loading DAW projects
-- Created: 2025-12-17

BEGIN;

-- Projects table for storing complete project state
CREATE TABLE IF NOT EXISTS projects (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    -- Musical settings
    bpm REAL NOT NULL DEFAULT 120.0,
    time_signature_numerator INTEGER NOT NULL DEFAULT 4,
    time_signature_denominator INTEGER NOT NULL DEFAULT 4,

    -- Complete project state serialized as JSON
    -- Includes: tracks, mixer state, automation, routing, effects, etc.
    project_data JSONB NOT NULL,

    -- Version tracking for compatibility
    schema_version INTEGER NOT NULL DEFAULT 1,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Soft delete
    deleted_at TIMESTAMPTZ
);

-- Indexes for performance
CREATE INDEX idx_projects_name ON projects(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_updated ON projects(updated_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_created ON projects(created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_bpm ON projects(bpm) WHERE deleted_at IS NULL;

-- GIN index for JSONB queries (for searching within project_data)
CREATE INDEX idx_projects_data ON projects USING GIN (project_data);

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_projects_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the update function
CREATE TRIGGER trigger_update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_projects_updated_at();

-- Project snapshots for version history (optional, for future autosave feature)
CREATE TABLE IF NOT EXISTS project_snapshots (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    snapshot_data JSONB NOT NULL,
    snapshot_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_snapshots_project_id ON project_snapshots(project_id, created_at DESC);

-- Comments/notes on projects
CREATE TABLE IF NOT EXISTS project_comments (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    position_beats REAL,  -- Optional: timestamp in project where comment applies
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_comments_project_id ON project_comments(project_id, created_at DESC);

-- Project tags for organization
CREATE TABLE IF NOT EXISTS project_tags (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(project_id, tag_name)
);

CREATE INDEX idx_project_tags_project_id ON project_tags(project_id);
CREATE INDEX idx_project_tags_tag_name ON project_tags(tag_name);

COMMIT;
