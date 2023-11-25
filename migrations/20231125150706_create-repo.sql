-- Create the repository table via sqlite.
CREATE TABLE IF NOT EXISTS repository (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    slug TEXT NOT NULL
);
