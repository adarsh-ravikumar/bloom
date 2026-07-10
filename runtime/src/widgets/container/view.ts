import "./style.scss";

import { View } from "@/core/view";

export class ContainerView extends View<HTMLDivElement> {
  constructor() {
    super("div");

    this._dom.classList.add("container");
  }
}
