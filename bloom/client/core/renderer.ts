import type { GenericWidget } from "@core/widget";
import { kebabCase } from "@runtime/utils";

export abstract class Renderer<T extends HTMLElement, W extends GenericWidget> {
  public dom!: T;
  protected widget: W;

  constructor(tagname: keyof HTMLElementTagNameMap, widget: W) {
    this.dom = document.createElement(tagname) as T;
    this.widget = widget;

    this.dom.classList.add(kebabCase(widget.constructor.name));

  }

  public synchronize(): void {
    this.dom.classList.toggle("hidden", this.widget.props.hidden);
    this.dom.style.zIndex = `${this.widget.props.zIndex}`;

    this.dom.setAttribute("data-b-id", `${this.widget.id}`);
    this.dom.setAttribute("data-b-cid", `${this.widget.constructorID}`);

    (this.dom as any).__bloom = {
      widget: this.widget, id: this.widget.id, constructorID: this.widget.constructorID
    };
  };

  public get classList(): DOMTokenList {
    return this.dom.classList;
  }

  public getAttribute(qualifiedName: string): string | null {
    return this.dom.getAttribute(qualifiedName);
  }

  public setAttribute(qualifiedName: string, value: any) {
    this.dom.setAttribute(qualifiedName, value)
  }
}


