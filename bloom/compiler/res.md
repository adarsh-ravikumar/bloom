An event is characterized by:

- Source ID (dynamic) => this tells me where in the map to look
- Event ID (static) => this tells me which event was invoked

on the UI end of things, you have:

- event fired = invoke the callback

# Problems

1) How does a widget remap an event on a child widget
2) How does a widget semantically raise an event on a child widget to its own scope
3) What does this event resolve to

# Cases

1) You semantically expose the event of a child.
 >
1) You remap the event to an expression.

 > the expression is the handler of that particular event call now.

1) You do NOT use an event on a child. => Don't Care

1) A widget A, when composing widget B, can choose to expose an event X on B

 > B.X becomes a semantic member of A.

1) A widget A, when composing widget B, can choose to remap an event X on B

 > B.X is no longer semantically available to any widget that composes A
 > This is the simplest case, where-in an event gets mapped to a local function or an event
