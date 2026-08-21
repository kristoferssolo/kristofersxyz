CREATE TABLE IF NOT EXISTS site (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    og_image TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    about TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profile_stack (
    item TEXT NOT NULL PRIMARY KEY,
    sort_order INTEGER NOT NULL,
    UNIQUE (sort_order)
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

CREATE TABLE IF NOT EXISTS working_principle (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    detail TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (label),
    UNIQUE (sort_order)
);

CREATE TABLE IF NOT EXISTS project (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (name),
    UNIQUE (sort_order)
);

CREATE TABLE IF NOT EXISTS project_stack (
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    item TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (project_id, sort_order),
    UNIQUE (project_id, item)
);

CREATE TABLE IF NOT EXISTS project_link (
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    PRIMARY KEY (project_id, sort_order),
    UNIQUE (project_id, label)
);

CREATE TABLE IF NOT EXISTS contact (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    body TEXT NOT NULL
);
