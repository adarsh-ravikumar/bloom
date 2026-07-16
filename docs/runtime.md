# Runtime

The Bloom runtime is responsible for executing widget trees, managing application state, and producing a rendered user interface.

Unlike traditional UI frameworks that tightly couple rendering with component logic, Bloom treats every widget as a composition of independent capabilities.

## Philosophy

Everything is a widget.

A widget is composed of zero or more of the following components:

- **View** — Responsible for rendering. Produces a DOM representation of the widget.
- **Behavior** — Defines how the widget reacts to user interaction and system events.
- **State** — Reactive data owned by the widget. Any change to state automatically schedules a rerender.

These components are intentionally independent. A widget may contain only a view, only behavior, only state, or any combination of the three.

This separation keeps widgets small, reusable, and predictable.

---

## Widget Tree

Every Bloom application is represented as a tree of widgets.

```text
Application
└── Window
    ├── Toolbar
    ├── SplitView
    │   ├── Layers
    │   └── Canvas
    └── Inspector
```

The runtime walks this tree to:

- create widgets
- update state
- dispatch events
- produce the rendered output

---

## Rendering

Rendering is declarative.

A widget does not manipulate the DOM directly. Instead, it returns a renderer describing what should exist. The runtime reconciles this renderer with the current DOM and applies only the required changes.

Widgets never own DOM nodes.

The runtime does.

---

## Behaviors

Behaviors encapsulate interaction.

Examples include:

- Mouse events
- Keyboard shortcuts
- Pointer capture
- Focus
- Drag & drop
- Context menus

Behaviors never render UI.

They only react to events and mutate state.

---

## State

State is the single source of truth.

Whenever state changes:

1. The state value is updated.
2. The runtime schedules a rerender.
3. The affected widgets are reconstructed.
4. The renderer applies the minimal DOM changes.

State should describe **what** exists, not **how** it is rendered.

---

## Serialization

Every widget can serialize its state.

```ts
interface Widget {

    serialize(): WidgetState;

    deserialize(state: WidgetState): void;

}
```

The runtime recursively walks the widget tree, producing a complete application snapshot.

This snapshot can be used for:

- Save / Load
- Undo / Redo
- Workspace restoration
- State synchronization
- Crash recovery

Serialization is a core runtime feature rather than an application concern.

---

## Renderer Independence

The runtime is renderer agnostic.

Widgets describe structure and behavior, but never depend on a specific rendering backend.

Current renderer:

- DOM

Future renderers may include:

- SVG
- Canvas
- Native
- Headless

Because rendering is separated from widget logic, the same widget can be reused across different rendering targets.

---

## Philosophy

The Bloom runtime is built around a few simple principles.

- Widgets should remain small and composable.
- Rendering and behavior are independent.
- State is reactive and serializable.
- The widget tree is the application.
- The runtime owns rendering.
- Applications describe intent rather than implementation.
