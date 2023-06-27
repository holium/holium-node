networking features, capabilities, utilities for holon nodes

- singleton WebSocket
- supports dedicated (single set of subscriptions) connection to Urbit
- supports N devices (clients)

```
                           <-----> device A
                        /
---------       --------
| Urbit | <---> | node | - <-----> device B
---------       --------
                        \
                           <-----> device C
```
