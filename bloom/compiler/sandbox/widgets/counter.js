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

    for (const subscriber of this._subscribers) {
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

    this.value = this._callback();

    Derived._current = previous;
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

class Parts {
  constructor(generator) {
    this.dom = document.createElement("span");

    this.generator = new Derived(generator);

    this.dom.innerText = this.generator.value;

    this.generator.subscribe((v) => {
      this.dom.innerText = v;
    })
  }
}

class Platform {
  constructor(tagName) {
    this.dom = document.createElement(tagName);
  }

  // hacky fix :salute
  get events() {
    return this;
  }

  on(ename, on) {
    this.dom.addEventListener(ename, on);
  }
}

class Text {
  constructor() {
    this.root = document.createElement("div");
    this.attrib = new State(undefined);
    this.value = new State(undefined);

    this.events = new EventMap([
      "hover", "click"
    ]);
  }

  hover(...params) {
    let value =
      (() => {
        console.log("hmmm... wdym hover?");
      })(...params);

    this.events.emit("hover", value);

    return value;
  }

  click(...params) {
    let value =
      (() => {
        console.log("Me was the clicked!");
        return this.value.value;
      })(...params);

    this.events.emit("click", value);

    return value;
  }

  build() {
    const w1 = new Platform("button");

    w1.events.on("click", (e) => {
      /* pattern is
       * (
       *   THE EXPRESSION AS IS (ofc after rewriting globals and what not)
       * )(e)
        */
      (() => {
        this.value.value++;
        this.click();
      })(e);
    });

    w1.events.on("hover", (e) => {
      (() => {
        this.attrib.value += "a";
        this.hover();
      })(e)
    });

    const w2 = new Parts(() => "Click me?");

    w1.addChild(w2);
    this.root.addChild(w1);

    const w3 = new Platform("p");

    const w4 = new Slot(undefined);
    this.slot = w4;

    // const w4 = new Parts(() => `Hello, world! ${this.attrib.value}\n\t${this.value.value}`)
    //
    // const w4 = new Parts([
    //   `Hello, world! `,
    //   new Derived(() => `${this.attrib.value}`),
    //   `\n\t`,
    //   new Derived(() => `${this.value.value}`),
    // ])
    //
    // w3.addChild(w4);
    this.root.addChild(w3);
  }
}

class Button {
  constructor({ uhh }) {
    this.uhh = new State(uhh);

    this.root = document.createElement("div");

    this.abc = undefined;

    this.events = new EventMap([
      "tap", "hover", "this_is_an_id_hover"
    ]);
  }

  hover(...params) {
    let value =
      (() => {
        console.log("hover again?");
      })(...params);

    this.events.emit("hover", value);

    return value;
  }

  testing(...params) {
    let value =
      ((count) => {
        console.log(count)
      })(...params);

    this.events.emit("testing", value);

    return value;
  }

  this_is_an_id_hover(e) {
    // exposed events are dynamically generated, and hence can't be called anywhere
    this.events.emit("this_is_an_id_hover", e);
    return e;
  }

  build() {
    const w1 = new Platform("button");

    w1.events.on("click", (e) => {
      (
        this.tap
      )(e)
    });

    w1.on("hover", (e) => {
      // exposed events are equivalent to writing 
      // a function name in the expression, with the only difference
      // being the function name is derived by the compiler
      (
        this.this_is_an_id_hover
      )(e)
    });

    const w2 = new Text();
    this.abc = w2.value;

    w2.events.on("hover", (e) => {
      (
        this.this_is_an_id_hover
      )(e)
    })

    w2.events.on("click", (e) => {
      (
        this.testing
      )(e)
    })
  }
}

