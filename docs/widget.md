# Widget Lifecycle

## Philsophy

> **Widgets describe *what* they are. The runtime decides *when* they update.**
>
> Widget authors build the tree, declare state, and define derived values. They never manually subscribe to state, manage dependency graphs, or trigger updates. The runtime discovers dependencies automatically and propagates changes through the reactive graph. This keeps widgets declarative while the runtime handles reactivity and synchronization.

Every widget goes through the following lifecycle.

```text
Constructor
    │
    ▼
compose()
    │
    ▼
initProps()
    │
    ▼
mount()
    │
    ▼
Reactive updates...
```

## Constructor

Only allocate child objects and define the widget hierarchy.

```ts
constructor() {
    super();

    this.compose();

    this.initProps({
        ...
    });
}
```

`compose()` **must** happen before `initProps()`.

This guarantees every child referenced by computed states already exists.

---

## compose()

Construct child widgets and establish the widget tree.

```ts
this.button = new Button();
this.text = new Text();

this.button.compose(this.text);

super.compose(this.button);
```

Rules:

* Construct every child here.
* Attach behaviours here.
* Never initialize state here.
* Called exactly once.

---

## initProps()

Initializes the widget's reactive graph.

```ts
this.initProps({
    count: 0,

    square: new Computed(() => {
        return this.props.count ** 2;
    })
});
```

`initProps()` performs:

1. Creates backing `State`s.
2. Attaches widget subscriptions.
3. Creates computed dependency graph.
4. Computes initial computed values.

After this point the widget is fully reactive.

---

## mount()

Mounts the widget into the DOM.

* Behaviours are attached.
* Children are mounted.
* DOM hierarchy is created.

After mounting, state changes automatically update the renderer.

---

# Reactive Model

Every property is internally represented as a `State`.

```ts
this.props.count
```

is conceptually

```text
Proxy
    │
    ▼
State<number>
```

The widget never interacts with `State` directly.

---

# Computed

Computed values derive state.

```ts
square: new Computed(() => {
    return this.props.count ** 2;
})
```

Dependencies are discovered automatically.

Reading

```ts
this.props.count
```

inside a `Computed` subscribes it to `count`.

No dependency arrays are required.

---

# Computed Rules

A computed represents a pure function.

Good:

```ts
new Computed(() => {
    return this.props.count ** 2;
});
```

Bad:

```ts
new Computed(() => {
    this.props.count++;
});
```

Bad:

```ts
new Computed(() => {
    this.button.props.hidden = true;
});
```

A computed **must not mutate state**.

Its only responsibility is returning a value.

Future versions of Bloom may enforce this at runtime.

---

# State Mutation

State is mutated through the proxy.

```ts
this.props.count++;
```

Never manipulate `State` objects directly.

---

# update()

`update()` synchronizes widget state with the renderer.

```ts
public override update() {
    this.text.props.value = `${this.props.count}`;

    super.update();
}
```

Rules:

* Read widget state.
* Update child widget properties.
* Do not mutate the widget's own state.
* Always call `super.update()`.

---

# Dependency Discovery

During evaluation of a computed

```ts
new Computed(() => {
    return this.props.count + this.props.other;
});
```

every property access is intercepted by the `Props` proxy.

Those properties automatically become dependencies.

---

# Widget Responsibilities

A widget owns:

* child hierarchy
* widget properties
* computed values
* update logic

A widget never owns:

* DOM manipulation
* subscription management
* dependency tracking

Those are handled by the runtime.

---

# Summary

| Method          | Responsibility                         |
| --------------- | -------------------------------------- |
| `constructor()` | Allocate widget                        |
| `compose()`     | Build widget tree                      |
| `initProps()`   | Initialize reactive state              |
| `mount()`       | Attach DOM & behaviours                |
| `update()`      | Synchronize state to children/renderer |

---
