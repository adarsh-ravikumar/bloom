import { Entity } from "./entity";
import type { Widget } from "./widget";

export class UIEvent extends Entity {
  protected _dom: HTMLElement;

  constructor(dom: HTMLElement) {
    super();

    this._dom = dom;
  }

  public synchronize(widget: Widget) {

  }

}
