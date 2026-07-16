import { Renderer } from "@core/renderer";

import "./style.scss";
import type { Container } from "./container";


export class ContainerRenderer extends Renderer<HTMLDivElement, Container> {
  constructor(widget: Container) {
    super("div", widget);

    this.dom.classList.add("container");
  }
}
