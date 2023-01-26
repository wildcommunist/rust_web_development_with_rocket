-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id     uuid primary key,
    name   varchar  not null,
    age    smallint not null default 0,
    grade  smallint not null default 0,
    active bool     not null default true
);

create index name_Active ON users (name, active);

INSERT INTO users
    (id, name, age, grade, active)
VALUES ('74d96050-8d8b-45e5-ac48-40c35208841e'::uuid,
        'Alexander Titarenko',
        36,
        1,
        true);