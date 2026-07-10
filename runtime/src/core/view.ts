import type { Widget } from "./widget";

export abstract class View<T extends HTMLElement> {
  protected _dom!: T;

  constructor(tagname: keyof HTMLElementTagNameMap) {
    this._dom = document.createElement(tagname) as T;
  }

  public synchronize(widget: Widget): void {
    widget.children.forEach((child) => {
      this.dom.appendChild(child.build().dom);
    })
  }

  public get dom(): T {
    return this._dom;
  }
}

