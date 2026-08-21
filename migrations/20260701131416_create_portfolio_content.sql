CREATE TABLE IF NOT EXISTS profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    display_name TEXT NOT NULL,
    legal_name TEXT,
    username TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    about TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS social_link (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    rel TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (label),
    UNIQUE (sort_order)
);

CREATE TABLE IF NOT EXISTS project (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (slug),
    UNIQUE (sort_order)
);

CREATE TABLE IF NOT EXISTS project_stack (
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    stack TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (project_id, sort_order),
    UNIQUE (project_id, stack)
);

CREATE TABLE IF NOT EXISTS project_link (
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (project_id, sort_order),
    UNIQUE (project_id, label)
);

CREATE TABLE IF NOT EXISTS working_principle (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    detail TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (label),
    UNIQUE (sort_order)
);
