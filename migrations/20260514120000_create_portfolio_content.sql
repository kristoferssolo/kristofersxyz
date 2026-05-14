CREATE TABLE profile (
    id BIGINT PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    about TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE social_links (
    id BIGSERIAL PRIMARY KEY,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    rel TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE projects (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    summary TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE project_stacks (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    stack TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE project_links (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    href TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE focus_areas (
    id BIGSERIAL PRIMARY KEY,
    label TEXT NOT NULL,
    detail TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);
