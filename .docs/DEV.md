# Getting Started

Guide that should be focused on how to get started as a developer looking to leverage holon features; e.g. websockets. For general installation, setup, and configuration please refer to the general README.md.

## Tips

- use the `trace` library for all terminal/cli logging

  - contains the following tracing macros which adds color coding and other contextual information to all printouts:

    - `trace_info`, `trace_info_ln`
    - `trace_warn`, `trace_warn_ln`
    - `trace_err` , `trace_err_ln`
    - `trace_good`, `trace_good_ln`

- For colored json output, always use the `trace_json` and `trace_json_ln` macros.

## Javascript

## websocket (ws) api

### create-room

action: `create-room`

```jsonc
{
  "type": "create-room",
  // unique id for the room
  "rid": "<id value>",
  // "media" | "background". defaults to "media", so you'll want to specify.
  "rtype": "<room type>",
  // user friendly title / display name for the room
  "title": "<title value>",
  // full path to the room (should be unique)
  "path": "<path value>"
}
```

The `create-room` sends the following reactions:

reaction: `rooms`
This reaction is sent to ALL peers.

action: `create-room`

```jsonc
{
  "type": "rooms",
  // array of "room" objects
  "rooms": [{
    // create-room rid
    "rid": "<id value>",
    // room type:
    //  "media" | "background"
    "rtype": "<room type>",
    // create-room title value
    "title": "<title>",
    // server id (e.g. patp) of creating ship
    "creator": "<creator>",
    // for now, always "default"
    "provider": "<provider>",
    // for now, "public"
    "access": "<access>",
    // array of peers (patp) currently in the room
    "present": ["patp", "patp", ...],
    // for now, empty array
    "whitelist": [],
    // for now, defaults to 10
    "capacity": 10,
    // optional value, but should match create-room path value
    "path": "<path>"
  }],
}
```

reaction: `room-created`
In addition to the `rooms` reaction which is sent to ALL peers, the `room-created` reaction is sent to the room creator.

```jsonc
{
  "type": "room-created",
  // create-room rid
  "rid": "<id value>",
  // room type:
  //  "media" | "background"
  "rtype": "<room type>",
  // create-room title value
  "title": "<title value>",
  // create-room path value
  "path": "<path value>"
}
```

### edit-room

This action is currently not used by the Realm desktop client.

action: `edit-room`
Some of the fields are optional. If they exist, they will overwrite the room values in the server store.

```jsonc
{
  "type": "edit-room",
  // room id of an existing room
  "rid": "<id value>",
  // optional title
  "title": "<title>",
  // option room access (e.g. public)
  "access": "<access>",
  // optional positive integer value specifying capacity
  "capacity": 1 // > 0
}
```

The `edit-room` sends the following reactions:

reaction: `edit-room`
This reaction is sent to ALL peers.

action: `edit-room`
There is no `rid` value sent to peers in response to edit-room. This seems like an oversight, but since this action is not utilized by the Realm desktop client, it is not an issue currently.

```jsonc
{
  "type": "edit-room",
  // room title (if provided in edit-room)
  "title": "<title>",
  // room access (if provided in edit-room)
  "access": "<access>",
  // room capacity (if provided in edit-room)
  "capacity": 1 // > 0
}
```

### delete-room

action: `delete-room`

```jsonc
{
  "type": "delete-room",
  // id value of an existing room
  "rid": "<id value>"
}
```

The `delete-room` sends the following reactions:

reaction: `room-deleted`
This reaction is sent to ALL peers.

```jsonc
{
  "type": "room-deleted",
  // delete-room rid value
  "rid": "<id value>"
}
```

### enter-room

action: `enter-room`

```jsonc
{
  "type": "enter-room",
  // id value of an existing room
  "rid": "<id value>"
}
```

The `enter-room` sends the following reactions:

reaction: `room-entered`
Note: this reaction is ONLY sent if you are not already in the room. If you are already in the room, the message is ignored and no reaction delivered to peers.

```jsonc
{
  "type": "room-entered",
  // enter-room rid value
  "rid": "<id value>",
  // peer_id - peer id of joining server (patp)
  "peer_id": "<peer_id>",
  // entered room metadata
  "room": {
    // room id
    "rid": "<id value>",
    // room type: 'media' | 'background'
    "rtype": "<room type>",
    // room title
    "title": "<title>",
    // room creator
    "creator": "<creator>",
    // room provider
    "provider": "<provider>",
    // room access
    "access": "<access>",
    // room peers
    "present": ["patp", "patp", ...],
    // room whitelist
    "whitelist": [],
    // room capacity
    "capacity": 1, // something > 0
    // room path
    "path": "<path>"
  },
}
```

### leave-room

action: `leave-room`

```jsonc
{
  "type": "leave-room",
  // id value of an existing room
  "rid": "<id value>"
}
```

The `leave-room` sends the following reactions:

reaction: `room-left`

```jsonc
{
  "type": "room-left",
  // leave-room rid value
  "rid": "<id value>",
  // peer_id - peer id of joining server (patp)
  "peer_id": "<peer_id>",
  // entered room metadata
  "room": {
    // room id
    "rid": "<id value>",
    // room type: 'media' | 'background'
    "rtype": "<room type>",
    // room title
    "title": "<title>",
    // room creator
    "creator": "<creator>",
    // room provider
    "provider": "<provider>",
    // room access
    "access": "<access>",
    // room peers
    "present": ["patp", "patp", ...],
    // room whitelist
    "whitelist": [],
    // room capacity
    "capacity": 1, // something > 0
    // room path
    "path": "<path>"
  },
}
```

### signal

action: `signal`
This is sent around to arbitrary peers to estabalish peer-to-peer connectivity between ships.

```jsonc
{
  "type": "signal",
  // peer where signal originated
  "from": "<from>",
  // destination peer
  "to": "<to>",
  // id value of an existing room
  "rid": "<id value>",
  // the original unmodified signal data forwarded via RTC signaling mechanism
  "signal": "<signal>"
}
```

The `signal` sends the following reactions:

reaction: `signal`
This reaction is sent to ALL peers.

```jsonc
{
  "type": "signal",
  // peer where signal originated
  "from": "<from>",
  // id value of an existing room
  "rid": "<id value>",
  // the original unmodified signal data forwarded via RTC signaling mechanism
  "signal": "<signal>"
}
```

### connect

action: `connect`
Send this on web socket `open` event to get the list of all rooms the provider is managing.

```jsonc
{
  "type": "connect"
}
```

The `connect` sends the following reactions:

reaction: `rooms`
This reaction is sent to the creator.

```jsonc
{
  "type": "rooms",
  // array of "room" objects
  "rooms": [{
    // room rid
    "rid": "<id value>",
    // room type: 'media' | 'background'
    "rtype": "<room type>",
    // room title
    "title": "<title>",
    // room creator
    "creator": "<creator>",
    // room provider
    "provider": "<provider>",
    // room access
    "access": "<access>",
    // array of peers (patp) currently in the room
    "present": ["patp", "patp", ...],
    // room whitelist
    "whitelist": [],
    // room capacity
    "capacity": 10,
    // room path value
    "path": "<path>"
  }],
}
```

### disconnect

action: `disconnect`

```jsonc
{
  "type": "disconnect",
  // id value of an existing room
  "rid": "<id value>"
}
```

The `disconnect` sends the following reactions:

reaction: `room-deleted`
This reaction is sent to ALL peers.

```jsonc
{
  "type": "room-deleted",
  // id value of an existing room
  "rid": "<id value>"
}
```
