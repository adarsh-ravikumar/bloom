import { Widget } from "../widget";

export class Button extends Widget {
  text: string;

  constructor(text: string) {
    super();
    this.text = text;
  }

  public update() {
    this.domHandle.innerHTML = `<button>${this.text}</button>`;
  }
}
