import { Renderer } from "@core/renderer";
import type { Text } from "./text";

import "./style.scss";

export class TextRenderer extends Renderer<HTMLSpanElement, Text> {
  constructor(widget: Text) {
    super("span", widget);

    this.dom.classList.add("text");
  }

  public synchronize() {
    this.dom.innerText = this.widget.props.value;
    this.dom.style.display = this.widget.props.isInline ? "inline" : "block";
  }
}
