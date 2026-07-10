import { View } from "@/core/view";

export class ButtonView extends View<HTMLButtonElement> {
  constructor() {
    super("button");

    this.dom.classList.add("button", "clickable");
  }
}
