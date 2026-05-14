CREATE TABLE profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    about TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE social_links (
id INTEGER PRIMARY KEY AUTOINCREMENT,
label TEXT NOT NULL,
href TEXT NOT NULL,
rel TEXT NOT NULL,
sort_order INTEGER NOT NULL
) ;

CREATE TABLE projects (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT NOT NULL,
summary TEXT NOT NULL,
sort_order INTEGER NOT NULL
) ;

CREATE TABLE project_stacks (
id INTEGER PRIMARY KEY AUTOINCREMENT,
project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
stack TEXT NOT NULL,
sort_order INTEGER NOT NULL
) ;

CREATE TABLE project_links (
id INTEGER PRIMARY KEY AUTOINCREMENT,
project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
label TEXT NOT NULL,
href TEXT NOT NULL,
sort_order INTEGER NOT NULL
) ;

CREATE TABLE focus_areas (
id INTEGER PRIMARY KEY AUTOINCREMENT,
label TEXT NOT NULL,
detail TEXT NOT NULL,
sort_order INTEGER NOT NULL
) ;

INSERT INTO profile (id, name, title, summary, about, email) VALUES
(
1,
'Kristofers Solo',
'Rust-focused software developer building reliable web systems and developer tools.',
'I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems, and tooling that makes day-to-day development simpler.',
'I focus on Rust and web systems where correctness, maintainability, and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools, and deployment surfaces that stay understandable over time.',
'mailto:dev@kristofers.xyz'
) ;

INSERT INTO social_links (label, href, rel, sort_order) VALUES
('Codeberg',
'https://codeberg.org/kristoferssolo',
'me noopener noreferrer',
1),
('GitHub', 'https://github.com/kristoferssolo', 'me noopener noreferrer', 2),
('Mastodon',
'https://fosstodon.org/@kristofers_solo',
'me noopener noreferrer',
3),
('Email', 'mailto:dev@kristofers.xyz', 'noopener noreferrer', 4) ;

INSERT INTO projects (id, name, summary, sort_order) VALUES
(
1,
'kristofers.xyz',
'A terminal-styled personal portfolio built with Rust, Leptos, Axum, server-side rendering, and SQLite-backed content.',
1
),
(
2,
'Rust Web Services',
'Backend and service work focused on typed APIs, clear operational boundaries, and maintainable deployment surfaces.',
2
),
(
3,
'Developer Tooling',
'CLI and automation work that keeps development workflows fast, explicit, and easy to reason about.',
3
) ;

INSERT INTO project_stacks (project_id, stack, sort_order) VALUES
(1, 'Rust', 1),
(1, 'Leptos', 2),
(1, 'Axum', 3),
(1, 'SQLite', 4),
(2, 'Rust', 1),
(2, 'Axum', 2),
(2, 'PostgreSQL', 3),
(2, 'Docker', 4),
(3, 'Rust', 1),
(3, 'CLI', 2),
(3, 'Automation', 3) ;

INSERT INTO project_links (project_id, label, href, sort_order) VALUES
(1, 'Codeberg', 'https://codeberg.org/kristoferssolo', 1),
(1, 'GitHub', 'https://github.com/kristoferssolo', 2) ;

INSERT INTO focus_areas (label, detail, sort_order) VALUES
(
'Rust web services',
'Backend systems with explicit data flow and predictable runtime behavior.',
1
),
(
'Typed interfaces',
'Small contracts that make invalid states harder to express.',
2
),
(
'Pragmatic testing',
'Coverage aimed at behavior, integrations, and regression-prone edges.',
3
),
(
'Maintainable deployment surfaces',
'Operational choices that are easy to inspect, document, and repeat.',
4
) ;
