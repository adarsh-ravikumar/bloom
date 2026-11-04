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
    this._subscribers.push(subscriber);
  }

  unsubscribe(subscriber) {
    const idx = this._subscribers.indexOf(subscriber);
    this._subscribers.splice(idx, 1);
  }
}

class Derived extends State {
  static _current;
  _states = new Set();

  constructor(callback) {
    super(undefined);
    this._callback = callback;
    this._onDelta = () => this.recompute();
    this.recompute();
  }

  recompute() {
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
  }

  appendChild(child) {
    this.root.appendChild(child.root)
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

  setAttribute(attribute, value) {
    this[attribute] = value
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

        part.subscribe((v) => {
          item.textContent = v;
        })
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
    this.root.className = "platform";
    this.slot = this;
  }

  // hacky fix :salute
  get events() {
    return this;
  }

  on(ename, on) {
    this.root.addEventListener(ename, on);
  }

  setAttribute(attribute, value) {
    if (value instanceof Derived) {
      this.root.setAttribute(attribute, value.value);
      value.subscribe((v) => {
        this.root.setAttribute(attribute, v);
      })
    }
  }
}


// bootstrap for testing purposes 
function bootstrap() {
  const b = new Button();
  b.build();

  document.querySelector("#app").appendChild(b.root);
}
