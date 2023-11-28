DEFINE TABLE projects SCHEMAFULL;

DEFINE FIELD project_name ON projects TYPE string;
DEFINE FIELD created_at ON projects TYPE datetime DEFAULT time::now();
