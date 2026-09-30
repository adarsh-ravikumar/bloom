# Bloom UI Engine Specification v1.0.0

## 1. Core UI Model

### 1.A) Workspace

This is the master construct for the entire application. Any and all UI elements
exist within (hierarchically) and are managed by the workspace. The workspace handles
resolution of commands and events.

### 1.B) Panel

A workspace is made up of one or more panels. A panel instance holds a unique,
developer-defined identifier. Multiple instances of a single panel may exist.

Fundamentally, there are two types of panels.

#### 1.B.1) Fixed Panel

This remains docked to the workspace. It cannot be undocked or moved by pointer
events. It can optionally be resized by pointer events. However, the panel may expose
commands that let it dock programmatically wherever needed.

##### 1.B.1.1 Example

A status bar always remains docked to the bottom, a sidebar always remains docked
to the left, and a menu bar remains docked to the top. The sidebar may need resizing
functionality, but the status bar and menu bar must stay fixed.

#### 1.B.2) Floating Panel

This panel may be undocked, moved around, resized, and docked wherever the user
pleases.

It exists in two major states:

- Docked   -> Attached to the main workspace
- Undocked -> Exists as a floating OS window that can be moved around independently,
              and may be docked back to the workspace as required

### 1.C) Widgets

Widgets are the atomic construct of Bloom UI. These are reusable components that
may be instanced multiple times in different panels.

Widgets model a contract that defines the command and event payloads.

Widgets, by definition, are "generic", in that they must maintain as much generality
as needed to solve a purpose. Hence, widget commands and events WILL lack semantics.

Therefore, no widget command or event is exposed directly. It must be managed by
the panel.

The panel models "semantic wrappers" around widget commands and events.

#### 1.C.1) Example

Say there exists a button widget that emits a "click" event. To the target of the
event, i.e. the client, a "button click" is ambiguous. There may be 10s and 100s
of buttons in the application, and managing them directly without any context
becomes cumbersome.

If there is a button instance in an inspector panel that, say, resets multiple
values, the panel will wrap this button's click event and extend it so as to
achieve the needed functionality, and expose a much better semantic event, say
"Inspector Reset".

The same goes for commands as well.

## 2) Core UI Semantics

### 2.A) Command Flow

When the workspace receives a command, it first uses the target identifier to
resolve the potential target. Then, the command identifier is passed on to
the target.

The target then uses the command identifier to resolve the right command, and
invokes it along with the deserialized payload.

The target may be the workspace itself (for commands like registering panels,
spawning and deleting panels, managing modals, etc.) or it may be a panel
instance.

### 2.B) Event Flow

When a source dispatches an event, it is bubbled up to the workspace
(if the source is not the workspace itself), and then dispatched outward
to the host.

### 2.C) Registration
