-- The versioned portfolio content. This is the source of truth the loader
-- reads, so it must stay in sync with the content model in src/app/content.rs.
-- Idempotent: it clears every table before inserting, so re-running it is safe.

BEGIN;

DELETE FROM project_link;
DELETE FROM project_stack;
DELETE FROM project;
DELETE FROM working_principle;
DELETE FROM social_link;
DELETE FROM profile_stack;
DELETE FROM profile;
DELETE FROM contact;
DELETE FROM site;

INSERT INTO site (id, url, title, description, og_image) VALUES (
    1,
    'https://kristofers.xyz/',
    'Kristofers Solo, Rust software developer',
    'I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems and tooling that makes day-to-day development simpler.',
    'https://kristofers.xyz/og.png'
);

INSERT INTO profile (id, name, title, summary, about, email) VALUES (
    1,
    'Kristofers Solo',
    'Rust-focused software developer building reliable web systems and developer tools.',
    'I build practical software with an emphasis on Rust, typed interfaces, maintainable web systems and tooling that makes day-to-day development simpler.',
    'I focus on Rust and web systems where correctness, maintainability and clear operational behavior matter. My preferred work is close to the boundary between product needs and engineering infrastructure: APIs, server-rendered applications, developer tools and deployment surfaces that stay understandable over time.',
    'mailto:dev@kristofers.xyz'
);

INSERT INTO profile_stack (sort_order, item) VALUES
    (1, 'Rust'),
    (2, 'Leptos'),
    (3, 'Axum'),
    (4, 'Tailwind');

INSERT INTO social_link (sort_order, label, href, rel) VALUES
    (1, 'Codeberg', 'https://codeberg.org/kristoferssolo', 'me noopener noreferrer'),
    (2, 'GitHub', 'https://github.com/kristoferssolo', 'me noopener noreferrer'),
    (3, 'Mastodon', 'https://fosstodon.org/@kristofers_solo', 'me noopener noreferrer'),
    (4, 'Email', 'mailto:dev@kristofers.xyz', 'noopener noreferrer');

INSERT INTO working_principle (sort_order, label, detail) VALUES
    (1, 'Rust web services', 'Backend systems with explicit data flow and predictable runtime behavior.'),
    (2, 'Typed interfaces', 'Small contracts that make invalid states harder to express.'),
    (3, 'Pragmatic testing', 'Coverage aimed at behavior, integrations and regression-prone edges.'),
    (4, 'Maintainable deployment surfaces', 'Operational choices that are easy to inspect, document and repeat.');

INSERT INTO project (id, sort_order, name, summary) VALUES
    (1, 1, 'guenther', 'Telegram bot that takes a social media link and sends the media back inline, so a shared post plays in the chat instead of opening a browser.'),
    (2, 2, 'traxor', 'Terminal UI for managing Transmission torrents: queue, inspect and control transfers without leaving the shell.'),
    (3, 3, 'cipher-workshop', 'Rust workspace implementing cipher algorithms, AES-128 and CBC among them, exposed through both a CLI and a web interface.');

INSERT INTO project_stack (project_id, sort_order, item) VALUES
    (1, 1, 'Rust'),
    (1, 2, 'Telegram'),
    (1, 3, 'yt-dlp'),
    (2, 1, 'Rust'),
    (2, 2, 'ratatui'),
    (2, 3, 'Transmission RPC'),
    (3, 1, 'Rust'),
    (3, 2, 'AES-128'),
    (3, 3, 'CLI'),
    (3, 4, 'WebAssembly');

INSERT INTO project_link (project_id, sort_order, label, href) VALUES
    (1, 1, 'GitHub', 'https://github.com/kristoferssolo/guenther'),
    (2, 1, 'Codeberg', 'https://codeberg.org/kristoferssolo/traxor'),
    (3, 1, 'GitHub', 'https://github.com/kristoferssolo/cipher-workshop');

INSERT INTO contact (id, name, body) VALUES (
    1,
    'Write to me',
    'Mail is the fastest route. Repositories and posts sit behind the links below.'
);

COMMIT;
