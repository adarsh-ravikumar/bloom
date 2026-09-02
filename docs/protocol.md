# Host-Client Protocol Specification v1.0.0

## Introduction

* This document describes the protocol between the Bloom host and a given
  Client SDK.

* This document is intended to be used as the canonical reference for Bloom, and
  be the first one to be updated in case of any changes in the protocol.

* Implementations MUST follow this specification and MUST NOT introduce protocol
  behaviour that is not defined by it

* The protocol version MUST be deliberately maintained by both the host and
  Client SDK implementations and MUST exactly match the version specified by
  this document

* The protocol itself assumes that the host has a canonical implementation that
  satisfies all invariants and paths prescribed by the protocol and is concrete.
  Hence, the failures are modelled around a client-facing error taxonomy

## 1. Packet Structure

A packet sent must follow the format:

```md
  ( 4 bytes) N
  ( 8 bytes) Status
  ( N bytes) Data
```

* N
  32 bit unsigned integer. Non-zero *(1I.A)*
  Number of bytes of data.
  NOTE: This does *NOT* include the 8 bytes for status

* Status
  An 8-byte UTF-8 encoded ASCII string. The value must be one of the statuses
  defined *(1I.B)*

* Data
  N bytes of intended data.

Note: All endian-sensitive fields must be transmitted / received in network-byte
      order

### Structures of the three packet classes

#### Request

```md
  ( 4 bytes) N
  ( 8 bytes) '0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00' ("REQUEST\0")
  ( N bytes) Data
```

#### Success

```md
  ( 4 bytes) N
  ( 8 bytes) '0x53 0x55 0x43 0x43 0x45 0x53 0x53 0x00' ("SUCCESS\0")
  ( N bytes) Data
```

#### Failure

```md
  (  4 bytes) 0x00 0x00 0x00 0x84                       (132)
  (  8 bytes) `0x46 0x41 0x49 0x4c 0x55 0x52 0x45 0x00` ("FAILURE\0")
  (  4 bytes) Error Code
  (128 bytes) Error Message
```

* Error Code: 32-bit unsigned integer
* Error Message: 128 byte UTF-8 encoded ASCII string (NUL-Padded)

### 1I) Invariants

#### 1I.A) Non-zero packet size

Any packet sent must contain data of non-zero, positive length.
This implies that a packet must at least be `13 bytes (4 + 8 + 1)` long.

The maximum length of data for a packet is `10 MiB (10,485,760 bytes)`.
This implies that a packet must at most be
`10,485,772 bytes (4 + 8 + 10,485,760)` long.

A packet outside this range is deemed invalid, and MUST NOT be transmitted.

#### 1I.B) Status

A packet may correspond to one of three statuses.

* If a packet semantically represents the act of initiating an action,
  then it is classified as a request packet.

* If a packet semantically represents the act of responding to an action,
  then it is classified as a response packet.

* All request packets must carry the status
  "REQUEST\0" (`0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00`)

* A response packet must carry the status

  * "SUCCESS\0", (`0x53 0x55 0x43 0x43 0x45 0x53 0x53 0x00`)
    if the requested action succeeded
  * "FAILURE\0" (`0x46 0x41 0x49 0x4c 0x55 0x52 0x45 0x00`)
    if the requested action failed.

### 1S) Success States

#### 1S.A) Validity of a packet

A packet is considered valid if all invariants specified in *1I* hold true for it.

### 1F) Failure States

#### 1F.A) Invalid packet size

Any packet that violates *1I.A* is said to carry an invalid packet size, and must
be dropped. Method of reporting the error, or the decision to, is implementation
specific

#### 1F.B) Invalid status

A packet that does NOT carry one of the following byte-sequences is deemed invalid,
and must be dropped

* `0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00` ("REQUEST\0")
* `0x53 0x55 0x43 0x43 0x45 0x53 0x53 0x00` ("SUCCESS\0")
* `0x46 0x41 0x49 0x4c 0x55 0x52 0x45 0x00` ("FAILURE\0")

Method of reporting the error, or the decision to, is implementation specific.

---

## 2. Connection

The bloom client will send the following 58 byte connection packet to the host

```md
  ( 4 bytes) 0x00 0x00 0x00 0x2E                       (46)
  ( 8 bytes) 0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00   ("REQUEST\0")
  ( 3 bytes) Client Version
  ( 3 bytes) Protocol Version
  ( 8 bytes) Client ID
  (32 bytes) App name
```

* Client Version
  Semantic version of the client SDK in the order `[major, minor, patch]`,
  1 byte per component.

* Protocol Version
  Semantic version of the Bloom Protocol in the order `[major, minor, patch]`,
  1 byte per component.

* Client ID
  Random generated 8-byte UTF-8 encoded ASCII string, client generated.
  This is to identify different instances, if any are already connected. *(2I.A)*

* App Name
  32 byte UTF-8 encoded ASCII string (NUL-Padded). Name of the application,
  defined in `proj.bloom.yaml`. This exists purely to ensure that the client
  connects only to the  intended application

### 2I) Invariants

#### 2I.A) Number of active clients

Only one active client may be connected to the host at any given time.

#### 2I.B) Version resolution

The host may support an array of client SDK versions. The host must support only
a single version of the protocol. The host and the client must remain on the
exact same protocol versions.

### 2S) Success States

#### 2S.A) Connection Successful

Connection is considered successful, upon receiving a connection packet from the
host.

This packet contains the list of all events that can possibly be dispatched by the
host. This is an exhaustive list, and the client can treat registration of any event
not on the list as invalid.

```md
  ( 4 bytes) N = 24 + Σ(4 + Nᵢ)
  ( 8 bytes) 0x53 0x55 0x43 0x43 0x45 0x53 0x53 0x00           ("SUCCESS\0")
  (10 bytes) 0x43 0x4F 0x4E 0x4E 0x45 0x43 0x54 0x45 0x44 0x00 ("CONNECTED\0")
  ( 3 bytes) Host Version
  ( 7 bytes) 0x45 0x56 0x45 0x4E 0x54 0x53 0x00                ("EVENTS\0")
  ( 4 bytes) Number of events (n)
                [n times]
  ( 4 bytes) Length (N)
  ( N bytes) Event
```

* Host Version
  Semantic version of the host in the order `[major, minor, patch]`,
  1 byte per component.

* Number of events = n
  32-bit unsigned integer. There are exactly n events present.

* Length = N
  32-bit unsigned integer. Length of the event name to be read.

* Event
  Name of event, UTF-8 encoded ASCII string of length N bytes.
  NUL-termination of the string is not required

### 2F) Failure States

A failure state is identified by receiving a failure packet of the following format

```md
  (  4 bytes) 0x00 0x00 0x00 0x84                       (132)
  (  8 bytes) `0x46 0x41 0x49 0x4c 0x55 0x52 0x45 0x00` ("FAILURE\0")
  (  4 bytes) Error Code
  (128 bytes) Error Message
```

For the "Connection" class of failures, it is expected that
`Error Code ϵ [100, 200)`

#### 2F.A) Host Inactive (Error Code: 100)

This occurs when the host has not bound to a socket at port 31415.

##### 2F.A Potential Sources (Non-exhaustive)

2F.A.1) Client running independently, without being managed by bloom itself.
2F.A.2) Port 31415 being used by another application.

##### 2F.A Potential Fixes

2F.A.1) The client must be run and managed by bloom only. Refer documentation.
2F.A.2) Set a port known to be available in the `proj.bloom.yaml` file.

#### 2F.B) Host Busy (Error Code: 101)

This occurs when a client is already connected to the bloom host.

##### 2F.B Potential Sources (Non-exhaustive)

2F.B.1) Client running independently, without being managed by bloom itself.
2F.B.2) Client instance surviving after an unexpected crash.

##### 2F.B Potential Fixes

2F.B.1) The client must be run and managed by bloom only. Refer documentation.
2F.B.2) Perform a clean restart of bloom.

#### 2F.C) Unsupported client version (Error Code: 102)

This occurs when the client SDK's version does not satisfy the host's client
version requirement. The host may support multiple / range of client versions.

#### 2F.D) Unsupported protocol version (Error Code: 103)

This occurs when the client SDK's protocol version does not satisfy the host's
protocol version requirement. The host supports exactly one version of the protocol.

#### 2F.E) Invalid App Name (Error Code: 105)

This occurs when the app name provided by the client does not match the app name
configured in `proj.bloom.yaml`

---

## 3. Commands

* A command is a request sent by the client to the host.
* The stream in which the commands are subsequently received and processed by the
  host is termed the "Inbound stream". This is derived from the perspective of the
  host, where the commands are inbound from the client

The general structure of a command packet is:
`[ Length ][ REQUEST\0 ][ FRONT\0 / HOST\0][ Identifiers ][ Payload ]`

The payload MUST match the schema defined by the user in the contract *(3I.C)*

There are primarily two classes of commands.

### Frontend Command

These commands are sent with intent to notify / modify frontend state.

```md
  (  4 bytes) 198 + n = N
  (  8 bytes) 0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00   ("REQUEST\0")
  (  6 bytes) 0x46 0x52 0x4F 0x4E 0x54 0x00             ("FRONT\0"  )
  (128 bytes) Target Identifier
  ( 64 bytes) Command Identifier
  (  n bytes) Payload
```

* Target Identifier
  128-byte UTF-8 encoded NUL-Padded ASCII string.
  This is unique to the destination on which the command is to be invoked

* Command Identifier
  64-byte UTF-8 encoded NUL-Padded ASCII string satisfying the pattern `^[a-zA-Z][a-zA-Z0-9_]*$`

* Payload
  UTF-8 encoded serialized JSON

### Host Command

These commands are sent with intent to register client-managed resources such as
panel instances to the host

```md
  (  4 bytes) 69 + n = N
  (  8 bytes) 0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00   ("REQUEST\0")
  (  5 bytes) 0x48 0x4F 0x53 0x54 0x00                  ("HOST\0"  )
  ( 64 bytes) Command Identifier
  (  n bytes) Payload
```

* Command identifier
  64-byte NUL-Padded UTF-8 encoded ASCII string satisfying the pattern `^[a-zA-Z][a-zA-Z0-9_]*$`

* Payload
  UTF-8 encoded serialized JSON

### 3I) Invariants

#### 3I.A) Packet Status

The status of a command packet MUST always be:
`0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00` ("REQUEST\0")

#### 3I.B) Command Type

The command type MUST always either be:

* `0x48 0x4F 0x53 0x54 0x00`      ("HOST\0" )
* `0x46 0x52 0x4F 0x4E 0x54 0x00` ("FRONT\0")

#### 3I.C) Payload

* Payload must be valid UTF-8 serialized JSON.
* The payload for a particular command must match its payload schema defined
  in its contract

### 3S) Success States

The client MUSTN'T expect a SUCCESS response for a command that runs with no
error, nor should it expect a return value.

### 3F) Failure States

A failure state is identified by receiving a failure packet of the following format

```md
  (  4 bytes) 0x00 0x00 0x00 0x84                       (132)
  (  8 bytes) `0x46 0x41 0x49 0x4c 0x55 0x52 0x45 0x00` ("FAILURE\0")
  (  4 bytes) Error Code
  (128 bytes) Error Message
```

For the "Command" class of failures, it is expected that `Error Code ϵ [200, 300)`

#### 3F.A) Panel not found (200)

The host is unable to resolve the panel based on the provided identifier

#### 3F.B) Command not found (201)

The host is unable to resolve the command based on the provided identifier.

#### 3F.C) Corrupted payload (202)

The host cannot deserialize the payload.

#### 3F.D) Illegal payload (203)

The payload violates the interface defined in the contract.

---

## 4. Events

* An event is a request sent by the host to the client.
* The client is expected to have registered n subscribers (n can be 0) at the
  client level only. The subscribers mustn't be exposed to the host.
* The dispatching of an event is then modelled as notifying the subscribers.
* The protocol does not specify the implementation detail for the subscription model

```md
  (  4 bytes) 64 + n = N
  (  8 bytes) 0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00   ("REQUEST\0")
  (128 bytes) Source Identifier
  ( 64 bytes) Event Identifier
  (  n bytes) Payload
```

* Source identifier
  128-byte UTF-8 encoded NUL-Padded ASCII string.
  This is unique to the source from which the event was dispatched

* Event Identifier
  64-byte NUL-Padded UTF-8 encoded ASCII string satisfying the pattern `^[a-zA-Z][a-zA-Z0-9_]*$`

* Payload
  UTF-8 encoded serialized JSON

### 4I) Invariants

#### 4I.A) Packet Status

The status of an event packet MUST always be:
`0x52 0x45 0x51 0x55 0x45 0x53 0x54 0x00` ("REQUEST\0")

#### 4I.B) Payload

* Payload must be valid UTF-8 serialized JSON.
* The payload's shape is modelled in the contract. The client must expect data
  of the same structure.

### 4S) Success States

The host DOESN'T expect a SUCCESS response for an event, nor should the client
attempt to provide one

### 4F) Failure States

No event-level protocol errors can occur. If the client registers invalid events,
they will simply never be invoked.

The client SDK is expected to warn the developer, as an exhaustive list of events
is provided at connection. Although, as far as the host is concerned, dormant events
do not exist. It is the client SDK's responsibility to handle the error reporting.

---

## Version History

### v1.0.0

* Initial specification.
