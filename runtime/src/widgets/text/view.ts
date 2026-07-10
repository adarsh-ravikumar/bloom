import "./style.scss";
import { View } from "@/core/view";
import type { Text } from "./text";

export class TextView extends View<HTMLSpanElement> {
  constructor() {
    super("span");

    this._dom.classList.add("text");
  }

  public override synchronize(widget: Text) {
    this.dom.innerText = widget.value;
    this.dom.style.display = widget.isInline ? "inline" : "block";
  }
}
