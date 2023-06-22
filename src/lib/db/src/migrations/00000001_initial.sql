create table if not exists chat_messages
(
    path         TEXT    not null,
    msg_id       TEXT    NOT NULL,
    msg_part_id  INTEGER NOT NULL,
    content_type TEXT,
    content_data TEXT,
    reply_to     TEXT,
    metadata     text,
    sender       text    NOT NULL,
    updated_at   INTEGER NOT NULL,
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER,
    received_at   INTEGER NOT NULL
);

create unique index if not exists chat_messages_path_msg_id_msg_part_id_uindex
    on chat_messages (path, msg_id, msg_part_id);


create table if not exists chat_paths
(
    path                        TEXT NOT NULL,
    type                        TEXT NOT NULL,
    metadata                    TEXT,
    invites                     TEXT default 'host' NOT NULL,
    peers_get_backlog           INTEGER default 1 NOT NULL,
    pins                        TEXT,
    max_expires_at_duration     INTEGER,
    pinned                      INTEGER default 0 NOT NULL,
    muted                       INTEGER default 0 NOT NULL,
    updated_at                  INTEGER NOT NULL,
    created_at                  INTEGER NOT NULL,
    received_at                 INTEGER NOT NULL
);

create unique index if not exists chat_paths_path_uindex
    on chat_paths (path);

CREATE TABLE IF NOT EXISTS chat_paths_flags
(
    path             TEXT NOT NULL,
    pinned           INTEGER default 0 NOT NULL,
    muted            INTEGER default 0 NOT NULL
);

create table if not exists chat_peers
(
    path        TEXT NOT NULL,
    ship        text NOT NULL,
    role        TEXT default 'member' NOT NULL,
    updated_at  INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    received_at  INTEGER NOT NULL
);

create unique index if not exists chat_peers_path_ship_uindex
    on chat_peers (path, ship);

create table if not exists chat_delete_logs
(
    change        TEXT NOT NULL,
    timestamp  INTEGER NOT NULL
);

create unique index if not exists chat_delete_logs_change_uindex
    on chat_delete_logs (timestamp, change);
