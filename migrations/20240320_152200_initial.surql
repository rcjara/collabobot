DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD username ON user TYPE string;
DEFINE FIELD zulip_username ON user TYPE string;
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();


DEFINE TABLE topic SCHEMAFULL;

DEFINE FIELD name ON topic TYPE string;
DEFINE FIELD created_at ON topic TYPE datetime DEFAULT time::now();


DEFINE TABLE technology SCHEMAFULL;

DEFINE FIELD name ON technology TYPE string;
DEFINE FIELD created_at ON technology TYPE datetime DEFAULT time::now();


DEFINE TABLE offer SCHEMAFULL;

DEFINE FIELD topics ON offer TYPE array<record<topics>>;
DEFINE FIELD technologies ON offer TYPE array<record<technologies>>;
DEFINE FIELD created_by ON offer TYPE record<users>;
DEFINE FIELD created_at ON offer TYPE datetime DEFAULT time::now();


DEFINE TABLE project SCHEMAFULL;

DEFINE FIELD name ON project TYPE string;
DEFINE FIELD description ON project TYPE string;
DEFINE FIELD github_link ON project TYPE string;
DEFINE FIELD topics ON offer TYPE array<record<topics>>;
DEFINE FIELD technologies ON offer TYPE array<record<technologies>>;
DEFINE FIELD created_by ON offer TYPE record<users>;
DEFINE FIELD created_at ON project TYPE datetime DEFAULT time::now();


DEFINE TABLE availability SCHEMAFULL;

DEFINE FIELD subject ON availability TYPE record<project | offer>;
DEFINE FIELD window ON availability TYPE string;
DEFINE FIELD location ON availability TYPE string;
DEFINE FIELD start_time ON availability TYPE datetime DEFAULT time::now();
DEFINE FIELD expiration ON availability TYPE datetime DEFAULT time::now() + 3d;
DEFINE FIELD created_at ON availability TYPE datetime DEFAULT time::now();
