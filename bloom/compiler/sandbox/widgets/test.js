

class Text {
  constructor() {
    this.value = new State(undefined);
    this.attribThatMustFold = undefined;
    this.events = new EventMap(["long_event_name", "focus", "hover",]);

  }
  test() {
    console.log("haha normie");
  }
  longEventName(...params) {
    const value = ((param1, param2, param3: number, param4: boolean, ...rest) => {
      console.log("looooonggg");
    }
    )(...params);
    this.events.emit("long_event_name", value);
    return value;
  }
  focus(...params) {
    const value = (() => {
      console.log("text focus");
      return this.value.value;
    }
    )(...params);
    this.events.emit("focus", value);
    return value;
  }
  hover(...params) {
    const value = (() => {
      console.log("text hover");
    }
    )(...params);
    this.events.emit("hover", value);
    return value;
  }

  build() {
    const w1 = new Platform("button");
    w1.events.on("click", (e) => {
      (

        (e) => {
          this.value.value++;
          click();
        }

      )(e);
    });
    w1.events.on("hover", (e) => {
      (

        (e) => {
          attrib += "a";
          this.hover();
        }

      )(e);
    });
    const w2 = new Parts([`\n  Click me?\n`,]);
    w1.appendChild(w2);
    this.root.appendChild(w1);
    const w3 = new Platform("p");
    const w4 = new Parts([`\n  Hello, world! `, new Derived(() => `${attrib}`), new Derived(() => `${this.value.value}`),]);
    w3.appendChild(w4);
    this.root.appendChild(w3);

  }
}

class Button {
  constructor() {
    this.uhh = new State(undefined);
    this.events = new EventMap(["renamed_hover", "text_focus", "hover",]);

  }
  handleLong() {
    conosle.log("uhhhhhhh")
  }
  renamedHover(e) {
    this.events.emit("renamed_hover", e);
    return e;
  }
  textFocus(e) {
    this.events.emit("text_focus", e);
    return e;
  }
  hover(...params) {
    const value = (() => {
      console.log("hover again?");
    }
    )(...params);
    this.events.emit("hover", value);
    return value;
  }

  build() {
    const w1 = new Platform("button");
    w1.events.on("click", (e) => {
      (
        tap
      )(e);
    });
    w1.events.on("hover", (e) => {
      (
        this.hover
      )(e);
    });
    w1.testing = new Derived(() => 123);
    w1.lol = new Derived(() => `123?${123}`);
    w1.uhh = new Derived(() => this.uhh.value);
    const w3 = new Text();
    w3.templateId = "text";
    w3.events.on("text_focus", (e) => { this.textFocus(e) });
    w3.events.on("hover", (e) => {
      (
        renamedHover
      )(e);
    });
    w3.events.on("long_event_name", (e) => {
      (
        handleLong
      )(e);
    });
    const w4 = new Parts([` \n    Hello!\n  `,]);
    w3.appendChild(w4);
    w1.appendChild(w3);
    const w5 = new Parts([new Derived(() => `${uhm}`),]);
    w1.appendChild(w5);
    this.root.appendChild(w1);

  }
}

class Counter {
  constructor() {
    this.count = new State(0);
    this.events = new EventMap(["button_hover",]);

  }
  buttonHover(e) {
    this.events.emit("button_hover", e);
    return e;
  }

  build() {
    const w1 = new Column();
    const w2 = new Button();
    w2.templateId = "button";
    w2.events.on("click", (e) => {
      (
        () => this.count.value++
      )(e);
    });
    w2.events.on("button_text_focus", (e) => { this.buttonTextFocus(e) });
    w2.events.on("button_hover", (e) => { this.buttonHover(e) });
    const w3 = new Parts([`\n    Click Me!\n  `,]);
    w2.appendChild(w3);
    w1.appendChild(w2);
    const w4 = new Text();
    const w5 = new Parts([`Count is `, new Derived(() => `${this.count.value}`),]);
    w4.appendChild(w5);
    w1.appendChild(w4);
    this.root.appendChild(w1);

  }
}


class View {
  constructor() {
    this.events = new EventMap([]);

  }

  build() {
    const w1 = new Counter();
    w1.templateId = "counter";
    w1.events.on("counter_button_hover", (e) => { this.counterButtonHover(e) });
    w1.events.on("counter_button_focus", (e) => { this.counterButtonFocus(e) });
    this.root.appendChild(w1);

  }
}
