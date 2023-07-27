# Technical Specifications and Notes

## Design Goals

- seamless with the interface provided by the Urbit

  meaning: apps should be able to communicate/interface with Urbit entities using existing message protocols. i.e. holon must support all channeling message protocols defined in the %eyre external interface spec "as is".

## Routing Messages

For the sake of this guide, we will view messaging as "those communications" that take place using the messaging protocols found here:

https://developers.urbit.org/reference/arvo/eyre/external-api-ref

Under these protocols, devices interact with the Urbit network using its http/sse based channeling system.

Once a channel is established, devices can begin sending messages to Urbit. Note that each of these messages requires an id; a value to uniquely identify a message within the channel context (device<=>urbit data/transmission flow).

So, within the current Urbit ecosystem, the device-to-urbit association has always been 1-1.

With holon however, the relationship has been abstracted one level. Specifically, under Holon, there is one level of indirection between an originating device and the ship ultimately service the request. This indirection is the holon abstraction.

This creates a bit of a conundrum. Let's take a look at the following example:

```
[device-1] (msg.id=22)
          \                                      / - [moon]
            \              [       ]            /
                ------->   | holon | -- > [ship] --- [moon]
            /              [       ]            \
          /                                      \ - [moon]
[device-2] (msg.id=22)
```

Note in the example above that each device sends out message with ID of 22. Once a channel is established, most clients start sending messages at ID = 1, incrementing with each subsequent message. This means that at some point in time, if the channels are kept opened long enough, each device will send a message with ID = 22 into Urbit. This is perfectly valid under "normal" device/urbit communcations. This is not a problem as long as the two "identical" messages are sent outside a certain window of time.

The holon operates in a high traffic multi threaded environment. It can service multiple requests concurrently and in parallel. And this thruput has the _potential_ to cause problems.

The problem being that a chance exists; albeit small, that two messages with ID = 22 are ingested by holon and relayed to Urbit before Urbit can finish processing either or both of them. This is NOT a problem getting messages into Urbit; however it IS a problem when trying to route a channel message back to its originating device.

The message ID is _all_ that the holon has to work with. The holon services _all_ device messages on a single channel to the ship. When Urbit ultimately sends a response, the response ID will be _all_ the holon has to work with to determine the destination device.

### Solution 1 - Holon Managed Message Ids

One solution is to manage an in-memory map (serializable to disk at intervals in case of system outage) of message ids to holon managed ids / device pairs.

Taking the example above, two messages enter the holon with id of 22.

Holon:

1. Gets an new ID from the its message sequencer
2. Overrides device-x's message with this new ID
3. Creates an entry to map the holon ID to the original device-x ID (22)
4. Forwards the "new" message (containing the Holon ID) to Urbit
5. Gets a response from the ship
6. Uses the response.id value to lookup the map entry containing the original ID and device ID
7. With original ID and device ID, holon can reconstruct the "true" response message (with original device message ID) and also has the device ID which is used to find the socket sender with which to send the response.

### Solution 2 - Dedicated Device Subscriptions

Another solution would be that each device is assigned to its own dedicated ship channel when the websocket connection is first established. This raises a question of efficiency and scale given the future vision of Realm and # of devices we plan to support.

## Message Protocols

holon rules for messaging e.g. formats, data types, etc...

### Urbit

Within an Urbit context, all message protocols for interacting with a ship should follow the protocols set forth in the following document:

https://developers.urbit.org/reference/arvo/eyre/external-api-ref

### holon

holon supports interactions and features outside of Urbit.

```json
{}
```
