ALTER TABLE profile_stack RENAME TO profile_technology;
ALTER TABLE project_stack RENAME TO project_technology;

ALTER TABLE project RENAME COLUMN name TO slug;
ALTER TABLE project ADD COLUMN title TEXT NOT NULL DEFAULT 'Untitled project';
ALTER TABLE project ADD COLUMN description_markdown TEXT NOT NULL DEFAULT '# Project';

UPDATE project SET title = slug;
