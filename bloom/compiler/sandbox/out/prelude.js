class State {
  _value = undefined;
  _subscribers = [];

  constructor(init) {
    this._value = init;
  }

  setState(value) {
    this._value = value instanceof State ? value.value : value;
  }

  get value() {
    Derived.track(this);
    return this._value;
  }

  set value(v) {
    // if same object, don't update.
    if (Object.is(v, this._value)) return;

    this._value = v;

    for (const subscriber of [...this._subscribers]) {
      subscriber(this._value);
    }
  }

  subscribe(subscriber) {
    if (subscriber instanceof Derived) {
      if (subscriber._destroyed) return;
    }

    this._subscribers.push(subscriber);
  }

  unsubscribe(subscriber) {
    const idx = this._subscribers.indexOf(subscriber);

    if (idx !== -1) {
      this._subscribers.splice(idx, 1);
    }
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
    // clear states, as stale states may still exist
    for (const state of this._states)
      state.unsubscribe(this._onDelta)

    this._states.clear();

    const previous = Derived._current;

    Derived._current = this;

    try {
      this.value = this._callback();
    } finally {
      Derived._current = previous;
    }
  }

  dependsOn(state) {
    if (this._states.has(state)) return;

    this._states.add(state);
    state.subscribe(this._onDelta)
  }

  static track(state) {
    if (!Derived._current) return;

    // avoid cyclic dependency
    if (state == Derived._current) return;

    Derived._current.dependsOn(state);
  }


  destroy() {
    if (this._destroyed) return;
    this._destroyed = true;

    for (const state of this._states) {
      state.unsubscribe(this._onDelta);
    }

    this._states.clear();
    this._subscribers.length = 0;
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
    this.root.className = "slot";
  }

  build() {
    return this;
  }
}

class Parts extends Widget {
  constructor(parts) {
    super(false);
    this.root = document.createDocumentFragment();

    parts.forEach((part) => {
      const item = document.createElement("span");

      if (part instanceof Derived) {
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

  on(ename, on) {
    this.root.addEventListener(ename, on);

    return () => {
      this.root.removeEventListener(ename, on);
    }
  }

  setAttribute(attribute, value) {
    if (value instanceof Derived) {
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

class CaseContainer extends Widget {
  constructor() {
    super();
    this.root.className = "conditional";
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
    this.caseContainer = new CaseContainer();
    this.root = this.caseContainer.root;
  }

  case(condition, factory) {
    if (condition instanceof Derived) {
      const update = () => this.update();
      condition.subscribe(update)
      this._cleanup.push(() => condition.unsubscribe(update))
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

      if (condition instanceof Derived ? condition.value : condition) {
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
    this.caseContainer.logicalDestroy();
    this.caseContainer.instantiate(factory)
  }
}

// bootstrap for testing purposes 
function bootstrap() {
  const b = new Button();
  b.build();

  document.querySelector("#app").appendChild(b.root);
}
