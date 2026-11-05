class State {
  _value = undefined;
  _subscribers = [];
  _destroyed = false;

  _proxyCache = new WeakMap();
  _rawCache = new WeakMap();

  // Mutation batching
  _mutationDepth = 0;
  _pendingNotification = false;

  // Cache instrumented array methods.
  _arrayMethodCache = new WeakMap();

  static ARRAY_MUTATORS = new Set([
    "copyWithin",
    "fill",
    "pop",
    "push",
    "reverse",
    "shift",
    "sort",
    "splice",
    "unshift"
  ]);

  constructor(init) {
    this._value = this._wrap(init);
  }

  _wrap(value) {
    if (value instanceof State) {
      value = value.value;
    }

    if (value === null || typeof value !== "object") {
      return value;
    }

    // Already a proxy owned by this State.
    if (this._rawCache.has(value)) {
      return value;
    }

    // Already wrapped by this State.
    if (this._proxyCache.has(value)) {
      return this._proxyCache.get(value);
    }

    const proxy = new Proxy(value, {
      get: (target, property, receiver) => {
        // Track dependencies on both direct and nested property reads.
        Derived.track(this);

        const result = Reflect.get(target, property, receiver);

        // Instrument native array-mutating methods.
        if (
          Array.isArray(target) &&
          typeof property === "string" &&
          State.ARRAY_MUTATORS.has(property) &&
          typeof result === "function"
        ) {
          let methods = this._arrayMethodCache.get(target);

          if (!methods) {
            methods = new Map();
            this._arrayMethodCache.set(target, methods);
          }

          if (methods.has(property)) {
            return methods.get(property);
          }

          const wrapped = (...args) => {
            this._beginMutation();

            try {
              return Reflect.apply(result, receiver, args);
            } finally {
              this._endMutation();
            }
          };

          methods.set(property, wrapped);

          return wrapped;
        }

        return this._wrap(result);
      },

      set: (target, property, value) => {
        const newValue = this._unwrap(value);
        const descriptor = Reflect.getOwnPropertyDescriptor(
          target,
          property
        );

        // Avoid unnecessary notifications for unchanged data properties.
        if (
          descriptor &&
          "value" in descriptor &&
          Object.is(descriptor.value, newValue)
        ) {
          return true;
        }

        const success = Reflect.set(
          target,
          property,
          newValue,
          target
        );

        if (success) {
          this._markChanged();
        }

        return success;
      },

      deleteProperty: (target, property) => {
        // Only notify when an own property actually exists.
        if (!Object.hasOwn(target, property)) {
          return true;
        }

        const success = Reflect.deleteProperty(target, property);

        if (success) {
          this._markChanged();
        }

        return success;
      },

      defineProperty: (target, property, descriptor) => {
        // Unwrap assigned values in property descriptors.
        if ("value" in descriptor) {
          descriptor = {
            ...descriptor,
            value: this._unwrap(descriptor.value)
          };
        }

        const success = Reflect.defineProperty(
          target,
          property,
          descriptor
        );

        if (success) {
          this._markChanged();
        }

        return success;
      }
    });

    this._proxyCache.set(value, proxy);
    this._rawCache.set(proxy, value);

    return proxy;
  }

  _unwrap(value) {
    return this._rawCache.get(value) ?? value;
  }

  // -------------------------
  // Mutation batching
  // -------------------------

  _beginMutation() {
    this._mutationDepth++;
  }

  _endMutation() {
    this._mutationDepth--;

    if (
      this._mutationDepth === 0 &&
      this._pendingNotification
    ) {
      this._pendingNotification = false;
      this._notify();
    }
  }

  _markChanged() {
    if (this._mutationDepth > 0) {
      this._pendingNotification = true;
    } else {
      this._notify();
    }
  }

  // -------------------------
  // State updates
  // -------------------------

  _notify() {
    if (this._destroyed) return;

    for (const subscriber of [...this._subscribers]) {
      subscriber(this._value);
    }
  }

  setState(value) {
    this.value = value instanceof State
      ? value.value
      : value;
  }

  get value() {
    Derived.track(this);
    return this._value;
  }

  set value(value) {
    const raw = this._unwrap(value);

    if (Object.is(raw, this._unwrap(this._value))) {
      return;
    }

    this._value = this._wrap(raw);
    this._markChanged();
  }

  // -------------------------
  // Subscriptions
  // -------------------------

  subscribe(subscriber) {
    if (this._destroyed) return;

    if (
      subscriber instanceof State &&
      subscriber._destroyed
    ) {
      return;
    }

    if (!this._subscribers.includes(subscriber)) {
      this._subscribers.push(subscriber);
    }
  }

  unsubscribe(subscriber) {
    const idx = this._subscribers.indexOf(subscriber);

    if (idx !== -1) {
      this._subscribers.splice(idx, 1);
    }
  }

  destroy() {
    if (this._destroyed) return;

    this._destroyed = true;
    this._subscribers.length = 0;
    this._pendingNotification = false;
  }
}
class Derived extends State {
  static _current;

  _states = new Set();
  _destroyed = false;

  constructor(callback) {
    super(undefined);

    this._callback = callback;
    this._onDelta = () => this.recompute();

    this.recompute();
  }

  recompute() {
    if (this._destroyed) return;

    // Remove stale dependencies.
    for (const state of this._states) {
      state.unsubscribe(this._onDelta);
    }

    this._states.clear();

    const previous = Derived._current;
    Derived._current = this;

    try {
      const value = this._callback();

      // Don't wrap values that are already reactive proxies
      // owned by another State.
      this._value = value;
    } finally {
      Derived._current = previous;
    }

    this._notify();
  }

  dependsOn(state) {
    if (this._states.has(state)) return;

    this._states.add(state);
    state.subscribe(this._onDelta);
  }

  static track(state) {
    const current = Derived._current;

    if (!current || current === state) return;

    current.dependsOn(state);
  }

  destroy() {
    if (this._destroyed) return;

    super.destroy();

    for (const state of this._states) {
      state.unsubscribe(this._onDelta);
    }

    this._states.clear();
  }
}

class EventMap {
  _listeners = {};

  constructor(events) {
    this._events = events;
    for (const event of this._events)
      this._listeners[event] = [];
  }

  on(event, listener) {
    this._listeners[event].push(listener)

    return () => {
      let idx = this._listeners[event].indexOf(listener);

      if (idx !== -1) {
        this._listeners[event].splice(idx, 1);
      }
    }
  }

  emit(event, e) {
    for (const listener of this._listeners[event])
      listener(e)
  }
}

class Widget {
  constructor(buildRoot = true) {
    if (buildRoot) {
      this.root = document.createElement("div");
      this.root.className = "widget";
    }

    this.slot = {};

    this._cleanup = [];
    this._children = [];
    this._destroyed = false;
  }

  appendChild(child) {
    this.root.appendChild(child.root)
    this._children.push(child);
  }

  setAttribute(attribute, value) {
    this[attribute] = value;
  }

  build() { }

  logicalDestroy() {
    if (this._destroyed) {
      return;
    }

    this._destroyed = true;

    for (const child of this._children) {
      child.logicalDestroy();
    }

    for (const cleanup of this._cleanup) {
      cleanup();
    }

    this.slot = {};
    this._children.length = 0;
    this._cleanup.length = 0;
  }
}

class Slot extends Widget {
  constructor() {
    super();
    this.root.classList.add("slot");
  }
}

class Parts extends Widget {
  constructor(parts) {
    super(false);
    this.root = document.createDocumentFragment();

    parts.forEach((part) => {
      const item = document.createElement("span");

      if (part instanceof State) {
        item.textContent = part.value;

        const update = (v) => {
          item.textContent = v;
        };

        part.subscribe(update);

        this._cleanup.push(() => {
          part.unsubscribe(update);
          part.destroy();
        });
      } else {
        item.textContent = part;
      }

      this.root.appendChild(item);
    })
  }
}

class Platform extends Widget {
  constructor(tagName) {
    super(false);
    this.root = document.createElement(tagName);
    this.slot = this;
  }

  // hacky fix :salute
  get events() {
    return this;
  }

  get interface() {
    return this.root;
  }

  on(ename, on) {
    this.root.addEventListener(ename, on);

    return () => {
      this.root.removeEventListener(ename, on);
    }
  }

  setAttribute(attribute, value) {
    if (value instanceof State) {
      this.root.setAttribute(attribute, value.value);

      const update = (v) => {
        this.root.setAttribute(attribute, v);
      };
      value.subscribe(update)

      this._cleanup.push(() => {
        value.unsubscribe(update);
        value.destroy();
      })
    } else {
      this.root.setAttribute(attribute, value);
    }
  }
}

class BlockContainer extends Widget {
  constructor() {
    super();
    this.root.classList.add("block");
    this._destroyed = true;
    this.factory = undefined;
  }

  instantiate(factory) {
    if (!this._destroyed) {
      return;
    }

    this._destroyed = false;
    this.root.replaceChildren();

    this.factory = factory;

    if (this.factory) {
      this.factory(this);
    }
  }
}

class Conditional extends Widget {
  constructor() {
    super(false);

    this.factories = [];
    this.defaultFactory = undefined;
    this.conditions = [];
    this.active = undefined;

    this.container = new BlockContainer();
    this.root = this.container.root;
  }

  case(condition, factory) {
    if (condition instanceof State) {
      const update = () => this.update();
      condition.subscribe(update)
      this._cleanup.push(() => {
        condition.unsubscribe(update);
        condition.destroy();
      })
    }

    this.conditions.push(condition);
    this.factories.push(factory);
  }

  default(factory) {
    this.defaultFactory = factory;
  }

  update() {
    let selected = -1;
    for (let i = 0; i < this.conditions.length; i++) {
      const condition = this.conditions[i];

      if (condition instanceof State ? condition.value : condition) {
        selected = i;
        break;
      }
    }

    if (selected === this.active)
      return;

    const factory =
      (selected === -1)
        ? this.defaultFactory
        : this.factories[selected];

    this.active = selected;

    this.container.logicalDestroy();
    this.container.instantiate(factory)
  }
}

class Each extends Widget {
  constructor(key, iterable, factory) {
    super(false);

    this.iterable = iterable;
    this.factory = factory;

    this.reconcile = key === undefined ? "index" : "key";
    this.key = key;
    this.instances = new Map();

    if (this.iterable instanceof State) {
      const update = () => this.update();
      this.iterable.subscribe(update);

      this._cleanup.push(() => {
        this.iterable.unsubscribe(update);
      });
    }

    this.container = new BlockContainer();
    this.root = this.container.root;
  }

  createInstance(item) {
    const block = new BlockContainer();
    const context = new State(item);

    block.context = context;

    block.instantiate((root) => {
      this.factory(this, context, root);
    });

    return block;
  }

  indexReconcilation() {
    const newMap = new Map();

    const iterable = this.iterable instanceof State
      ? this.iterable.value
      : this.iterable;

    for (let i = 0; i < iterable.length; i++) {
      const item = iterable[i];
      const existing = this.instances.get(i);

      if (existing) {
        newMap.set(i, existing);
      } else {
        const instance = this.createInstance(item);
        newMap.set(i, instance);
      }
    }

    for (const [index, instance] of this.instances) {
      if (!newMap.has(index)) {
        instance.logicalDestroy();
      }
    }

    this.instances = newMap;

    let index = 0;

    for (const instance of this.instances.values()) {
      const current = this.container.root.children[index];

      if (current !== instance.root) {
        this.container.root.insertBefore(instance.root, current ?? null);
      }

      index++;
    }
  }

  keyReconcilation() {
    const newMap = new Map();

    const iterable = this.iterable instanceof State
      ? this.iterable.value
      : this.iterable;

    for (const item of iterable) {
      const key = this.key({ value: item });

      if (newMap.has(key)) {
        this.container.root.replaceChildren();
        // TODO: handle runtime errors thrown by bloom
        throw new Error(`Duplicate key in each block: ${key}`);
      }

      const existing = this.instances.get(key);

      if (existing) {
        existing.context.value = item;
        newMap.set(key, existing);
      } else {
        const instance = this.createInstance(item);
        newMap.set(key, instance);
      }
    }

    // Destroy instances that no longer exist.
    for (const [key, instance] of this.instances) {
      if (!newMap.has(key)) {
        instance.logicalDestroy();
        instance.root.remove();
      }
    }

    this.instances = newMap;

    // Reorder only nodes that are actually out of position.
    let index = 0;

    for (const instance of this.instances.values()) {
      const current = this.container.root.children[index];

      if (current !== instance.root) {
        this.container.root.insertBefore(
          instance.root,
          current ?? null
        );
      }

      index++;
    }
  }

  update() {
    if (this.reconcile === "index") {
      this.indexReconcilation();
    } else {
      this.keyReconcilation();
    }
  }
}
// bootstrap for testing purposes 
function bootstrap() {
  const b = new Todo();
  b.build();

  document.querySelector("#app").appendChild(b.root);
}
