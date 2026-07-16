import { Renderer } from "@core/renderer";

import "./style.scss";
import type { Button } from "./button";

export class ButtonRenderer extends Renderer<HTMLButtonElement, Button> {
  constructor(widget: Button) {
    super("button", widget);
  }
}
