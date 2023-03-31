-- Add migration script here
create table if not exists nuki_log(
    discord_uid text not null,
    count integer default 0 not null,
    timestamp datetime not null,
    comment text,
    primary key (discord_uid, timestamp)
);