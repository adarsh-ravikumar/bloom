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
class Container extends Widget {
constructor() {
super();
this.root.classList.add("b-container");
this.events = new EventMap([]);

}

build () {
const w1 = new Platform("div");
w1.setAttribute("style", new Derived(() => `outline: 2px solid limegreen; background: yellow;`));
this.slot["header"] = new Slot();
w1.appendChild(this.slot["header"].build());
this.appendChild(w1);

return this;}
}
class Text extends Widget {
constructor() {
super();
this.root.classList.add("b-text");
this.test = "";
this.color = new State("blue");
this.events = new EventMap([]);

}

build () {
const w1 = new Container();
w1.build();
const w2 = w1.slot["header"];
this.slot["header"] = new Slot();
w2.appendChild(this.slot["header"].build());
this.appendChild(w1);
const w4 = new Platform("div");
w4.setAttribute("test", new Derived(() => this.test));
w4.setAttribute("style", new Derived(() => `background-color: ${this.color.value}; color: white;`));
this.slot["counter"] = new Slot();
w4.appendChild(this.slot["counter"].build());
this.appendChild(w4);
const w7 = new Parts([`\n\nSome text in the middle cuz why not!\n\n`,]);
this.appendChild(w7);
const w8 = new Platform("div");
w8.setAttribute("style", new Derived(() => `background-color: red; color: white;`));
const w10 = new Parts([`\n  Hallo!\n`,]);
w8.appendChild(w10);
this.appendChild(w8);

return this;}
}
class Button extends Widget {
constructor() {
super();
this.root.classList.add("b-button");
this.textBackground = undefined;
this.count = new State(1);
this.events = new EventMap([]);

}

build () {
const w1 = new Platform("button");
w1.events.on("click", (e) => {(
() => {
  this.count.value ++;
  if (this.count.value % 2 == 0) {
      this.textBackground.value = "black";
  } else {
      this.textBackground.value = "blue";
  }
}
)(e);
});
const w2 = new Text();
w2.build();
this.textBackground = w2.color;
const w3 = w2.slot["counter"];
const w4 = new Parts([`\n      Counter: `,new Derived(() => `${this.count.value}`),]);
w3.appendChild(w4);
const w5 = w2.slot["header"];
const w6 = new Parts([`\n      magical!\n    `,]);
w5.appendChild(w6);
w1.appendChild(w2);
this.appendChild(w1);

return this;}
}
