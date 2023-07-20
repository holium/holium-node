create table if not exists packets
(
    /* auto increment ROWID */
    id           INTEGER PRIMARY KEY,
    /*
        full action payload
        @see: https://developers.urbit.org/reference/arvo/eyre/external-api-ref#actions
    */
    content      TEXT NOT NULL,
    /* when the holon actually received the packet on the websocket channel */
    received_at  INTEGER NOT NULL
);

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
    received_at  INTEGER NOT NULL
);

create unique index if not exists chat_messages_path_msg_id_msg_part_id_uindex
    on chat_messages (path, msg_id, msg_part_id);