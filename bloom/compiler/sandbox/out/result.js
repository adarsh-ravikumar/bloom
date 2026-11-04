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
      this.factory(context, root);
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
class Text extends Widget {
constructor() {
super();
this.root.classList.add("b-text");
this.events = new EventMap([]);
this.slot = new Slot();

}

build () {
const w1 = new Platform("p");
w1.appendChild(this.slot);
w1.build();
this.appendChild(w1);

}
}
class Column extends Widget {
constructor() {
super();
this.root.classList.add("b-column");
this.events = new EventMap([]);
this.slot = new Slot();

}

build () {
const w1 = new Platform("div");
w1.setAttribute("class", new Derived(() => `column`));
w1.setAttribute("style", new Derived(() => `display: flex; flex-direction: column; gap: 1rem;`));
w1.appendChild(this.slot);
w1.build();
this.appendChild(w1);

}
}
class Row extends Widget {
constructor() {
super();
this.root.classList.add("b-row");
this.events = new EventMap([]);
this.slot = new Slot();

}

build () {
const w1 = new Platform("div");
w1.setAttribute("class", new Derived(() => `row`));
w1.setAttribute("style", new Derived(() => `display: flex; gap: 1rem;`));
w1.appendChild(this.slot);
w1.build();
this.appendChild(w1);

}
}
class Button extends Widget {
constructor() {
super();
this.root.classList.add("b-button");
this.events = new EventMap(["click", ]);
this.slot = new Slot();

}
click(...params) {
const value = ((e) => {
    return e.button;
  }
)(...params);
this.events.emit("click", value);
return value;
}

build () {
const w1 = new Platform("button");
this._cleanup.push(w1.events.on("click", (e) => {(
(e) => this.click(e)
)(e);
}));
w1.appendChild(this.slot);
w1.build();
this.appendChild(w1);

}
}
class Input extends Widget {
constructor() {
super();
this.root.classList.add("b-input");
this.value = new State("");
this.events = new EventMap(["change", ]);

}
change(...params) {
const value = ((event) => {
    this.value.value = event.target.value;
  }
)(...params);
this.events.emit("change", value);
return value;
}

build () {
const w1 = new Platform("input");
this._cleanup.push(w1.events.on("change", (e) => {(
this.change
)(e);
}));
w1.setAttribute("type", new Derived(() => `text`));
w1.setAttribute("value", new Derived(() => this.value.value));
w1.build();
this.appendChild(w1);

}
}
class Task extends Widget {
constructor() {
super();
this.root.classList.add("b-task");
this.deleteItem = new State(undefined);
this.name = new State(undefined);
this.taskId = new State(undefined);
this.markDone = new State(undefined);
this.isEditing = new State(false);
this.status = new State(undefined);
this.editName = new State(undefined);
this.value = new State("");
this.events = new EventMap([]);

}
edit(){
    if (this.isEditing.value) {
      this.isEditing.value = false;
      this.editName.value(this.taskId.value, this.value.value);
      return;
    }

    this.isEditing.value = true;
  }

build () {
const w1 = new Row();
const w2 = w1.slot;
const condition3 = new Conditional();
condition3.case(new Derived(() => this.status.value == "done"), (w3) => {
const w4 = new Text();
const w5 = w4.slot;
const w6 = new Parts([` Task Completed! `,]);
w5.appendChild(w6);
w4.build();
w3.appendChild(w4);
});
condition3.default((w7) => {
const w8 = new Text();
const w9 = w8.slot;
const w10 = new Parts([` In Progress `,]);
w9.appendChild(w10);
w8.build();
w7.appendChild(w8);
});
w2.appendChild(condition3);
condition3.update();
const w11 = new Column();
const w12 = w11.slot;
const condition13 = new Conditional();
condition13.case(new Derived(() => this.isEditing.value), (w13) => {
const w14 = new Input();
this.value = w14.value;
w14.value.value = new Derived(() => this.name.value);
w14.build();
w13.appendChild(w14);
});
condition13.default((w15) => {
const w16 = new Text();
const w17 = w16.slot;
const condition18 = new Conditional();
condition18.case(new Derived(() => this.status.value == "done"), (w18) => {
const w19 = new Platform("s");
const w20 = new Parts([new Derived(() => `${this.name.value}`),]);
w19.appendChild(w20);
w19.build();
w18.appendChild(w19);
});
condition18.default((w21) => {
const w22 = new Parts([new Derived(() => `${this.name.value}`),]);
w21.appendChild(w22);
});
w17.appendChild(condition18);
condition18.update();
w16.build();
w15.appendChild(w16);
});
w12.appendChild(condition13);
condition13.update();
const w23 = new Text();
const w24 = w23.slot;
const w25 = new Parts([`(id = `,new Derived(() => `${this.taskId.value}`),`)`,]);
w24.appendChild(w25);
w23.build();
w12.appendChild(w23);
const w26 = new Parts([new Derived(() => `${this.status.value}`),]);
w12.appendChild(w26);
w11.build();
w2.appendChild(w11);
const w27 = new Button();
this._cleanup.push(w27.events.on("click", (e) => {(
this.edit
)(e);
}));
const w28 = w27.slot;
const condition29 = new Conditional();
condition29.case(new Derived(() => this.isEditing.value), (w29) => {
const w30 = new Parts([`\n      Save\n    `,]);
w29.appendChild(w30);
});
condition29.default((w31) => {
const w32 = new Parts([`\n      Edit\n    `,]);
w31.appendChild(w32);
});
w28.appendChild(condition29);
condition29.update();
w27.build();
w2.appendChild(w27);
const w33 = new Button();
this._cleanup.push(w33.events.on("click", (e) => {(
() => this.deleteItem.value(this.taskId.value)
)(e);
}));
const w34 = w33.slot;
const w35 = new Parts([`\n    Delete\n  `,]);
w34.appendChild(w35);
w33.build();
w2.appendChild(w33);
const w36 = new Button();
this._cleanup.push(w36.events.on("click", (e) => {(
() => this.markDone.value(this.taskId.value)
)(e);
}));
const w37 = w36.slot;
const w38 = new Parts([`\n    Mark Done\n  `,]);
w37.appendChild(w38);
w36.build();
w2.appendChild(w36);
w1.build();
this.appendChild(w1);

}
}
class Todo extends Widget {
constructor() {
super();
this.root.classList.add("b-todo");
this.task = new State(undefined);
this.items = new State([]);
this.events = new EventMap([]);

}
addItem(){
    let id = this.uuid4();
    this.items.value.push({id: id, task: this.task.value, status: undefined});
  }
markDone(id){
    let toMark = this.findItem(id);
    this.items.value[toMark].status = "done";
  }
uuid4(){
    return Array.from(
        { length: 4 },
        () => Math.floor(Math.random() * 16).toString(16)
    ).join(""); 
  }
deleteItem(id){
      let toDel = this.findItem(id);
      if (toDel != -1) this.items.value.splice(toDel, 1);
  }
editItem(id, name){
      let toEdit= this.findItem(id);
      this.items.value[toEdit].task = name;
  }
findItem(id){
      let idx = -1;
      for (let i = 0; i < this.items.value.length; i ++) {
        const task = this.items.value[i];

        if (task.id == id) {
           idx = i;
           break;
        }
      }

      return idx;
  }

build () {
const w1 = new Platform("h1");
const w2 = new Parts([`Todo`,]);
w1.appendChild(w2);
w1.build();
this.appendChild(w1);
const w3 = new Row();
const w4 = w3.slot;
const w5 = new Input();
this.task = w5.value;
w5.build();
w4.appendChild(w5);
const w6 = new Button();
this._cleanup.push(w6.events.on("click", (e) => {(
this.addItem
)(e);
}));
const w7 = w6.slot;
const w8 = new Parts([`Add Item`,]);
w7.appendChild(w8);
w6.build();
w4.appendChild(w6);
w3.build();
this.appendChild(w3);
const w9 = new Platform("h2");
const w10 = new Parts([`Tasks`,]);
w9.appendChild(w10);
w9.build();
this.appendChild(w9);
const w11 = new Column();
const w12 = w11.slot;
const each13 = new Each((task) => task.value.id, new Derived(() => this.items.value), (task, w13) => {
const w14 = new Task();
w14.setAttribute("name", new Derived(() => task.value.task));
w14.setAttribute("taskId", new Derived(() => task.value.id));
w14.setAttribute("status", new Derived(() => task.value.status));
w14.setAttribute("deleteItem", new Derived(() => (id) => this.deleteItem(id)));
w14.setAttribute("markDone", new Derived(() => (id) => this.markDone(id)));
w14.setAttribute("editName", new Derived(() => (id, name) => this.editItem(id, name)));
w14.build();
w13.appendChild(w14);
});
w12.appendChild(each13);
each13.update();
w11.build();
this.appendChild(w11);

}
}
