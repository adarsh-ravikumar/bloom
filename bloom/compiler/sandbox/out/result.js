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
this.color = new State("blue");
this.test = "";
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
const w1 = new Platform("h1");
const w2 = new Parts([`\n  Counter:  `,new Derived(() => `${this.count.value}`),]);
w1.appendChild(w2);
this.appendChild(w1);
const w3 = new Platform("button");
this._cleanup.push(w3.events.on("click", (e) => {(
() => {
  this.count.value ++;
  if (this.count.value % 2 == 0) {
      this.textBackground.value = "black";
  } else {
      this.textBackground.value = "blue";
  }
}
)(e);
}));
const w4 = new Text();
w4.build();
this.textBackground = w4.color;
const w5 = w4.slot["counter"];
const w6 = new Parts([`\n      Counter: `,new Derived(() => `${this.count.value}`),]);
w5.appendChild(w6);
const w7 = w4.slot["header"];
const w8 = new Parts([`\n      magical!\n    `,]);
w7.appendChild(w8);
w3.appendChild(w4);
this.appendChild(w3);
const w9 = new Platform("div");
const w10 = new Platform("h3");
const w11 = new Parts([`\n    Fizz or Buzz?\n  `,]);
w10.appendChild(w11);
w9.appendChild(w10);
const condition12 = new Conditional();
condition12.case(new Derived(() => this.count.value % 3 == 0 && this.count.value % 5 == 0), (w12) => {
const w13 = new Platform("h1");
const w14 = new Parts([`FizzBuzz!!`,]);
w13.appendChild(w14);
w12.appendChild(w13);
const condition15 = new Conditional();
condition15.case(new Derived(() => this.count.value == 3), (w15) => {
const w16 = new Platform("h2");
const w17 = new Parts([`The count is 3`,]);
w16.appendChild(w17);
w15.appendChild(w16);
});
condition15.case(new Derived(() => this.count.value == 5), (w18) => {
const w19 = new Platform("h2");
const w20 = new Parts([`The count is 5`,]);
w19.appendChild(w20);
w18.appendChild(w19);
});
w12.appendChild(condition15);
condition15.update();
});
condition12.case(new Derived(() => this.count.value % 3 == 0), (w21) => {
const w22 = new Platform("h1");
const w23 = new Parts([`Fizz!`,]);
w22.appendChild(w23);
w21.appendChild(w22);
const condition24 = new Conditional();
condition24.case(new Derived(() => this.count.value == 3), (w24) => {
const w25 = new Platform("h2");
const w26 = new Parts([`The count is 3`,]);
w25.appendChild(w26);
w24.appendChild(w25);
});
w21.appendChild(condition24);
condition24.update();
});
condition12.case(new Derived(() => this.count.value % 5 == 0), (w27) => {
const w28 = new Platform("h1");
const w29 = new Parts([`Buzz!`,]);
w28.appendChild(w29);
w27.appendChild(w28);
const condition30 = new Conditional();
condition30.case(new Derived(() => this.count.value == 5), (w30) => {
const w31 = new Platform("h2");
const w32 = new Parts([`The count is 5`,]);
w31.appendChild(w32);
w30.appendChild(w31);
});
w27.appendChild(condition30);
condition30.update();
});
condition12.default((w33) => {
const w34 = new Platform("h1");
const w35 = new Parts([` Neither :( `,]);
w34.appendChild(w35);
w33.appendChild(w34);
});
w9.appendChild(condition12);
condition12.update();
this.appendChild(w9);

return this;}
}
