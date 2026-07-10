import type { GenericWidget } from "./widget";

export abstract class View<T extends HTMLElement> {
  protected _dom!: T;

  constructor(tagname: keyof HTMLElementTagNameMap) {
    this._dom = document.createElement(tagname) as T;
  }

  public synchronize(widget: GenericWidget): void { };

  public get dom(): T {
    return this._dom;
  }

  public get classList(): DOMTokenList {
    return this._dom.classList;
  }
}

