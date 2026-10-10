// @ts-nocheck

class Button {
  dom: HTMLButtonElement;
  id: number;

  internal: number = 0;

  eventMap: object;

  constructor(id: number) {
    this.id = id;
    this.dom = document.createElement("button");
    this.dom.innerText = `Button: ${this.id}`
    document.body.appendChild(this.dom);

    this.eventMap = { "click": [], "even": [] };

    this.dom.addEventListener("click", () => this.click())
  }

  click() {
    let listeners = this.eventMap["click"];
    this.internal++;

    for (const listener of listeners) {
      listener();
    }

    if (this.internal % 2 == 0) {
      this.even();
    }
  }

  even() {
    let listeners = this.eventMap["even"];

    for (const listener of listeners) {
      listener();
    }
  }

  on(event: string, callback) {
    this.eventMap[event].push(callback)
  }
}

class Counter {
  count: number = 0;

  id: number;
  text: HTMLParagraphElement;
  evenText: HTMLParagraphElement;

  constructor(id: number) {
    this.id = id;

    const button = new Button(this.id);

    this.text = document.createElement('p');
    document.body.appendChild(this.text);

    this.evenText = document.createElement('p');
    document.body.appendChild(this.evenText);
    this.evenText.innerText = `${this.id} IS EVEN!!`;

    this.updateCount();


    button.on("click", () => {
      this.evenText.style.opacity = 0;
      this.increment();
      this.updateCount();
    })

    button.on("even", () => {
      this.evenText.style.opacity = 1;
    })
  }

  increment() {
    this.count++;
  }

  updateCount() {
    this.text.innerText = `Count of ${this.id} = ${this.count} `;
  }
}

// create counter!
new Counter(0);
new Counter(1); 
