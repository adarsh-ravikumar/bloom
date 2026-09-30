import { IDPool } from "./id";

export interface WidgetTemplate {
  widget: Widget;
  render(): void;
}

export class Widget {
  readonly id: number;
  readonly domHandle: HTMLDivElement;

  private children: Widget[] = [];

  private template: WidgetTemplate;

  constructor(template: WidgetTemplate) {
    this.id = IDPool.acquire();
    this.domHandle = document.createElement("div");
    this.template = template;
    this.template.widget = this;

    this.domHandle.className = "widget";
    this.domHandle.id = `widget-${this.id.toString().padStart(5, "0")}`;
    this.template.render();
  }

  append(child: Widget) {
    this.children.push(child);
    this.domHandle.appendChild(child.domHandle);
  }

  remove(child: Widget) {
    const index = this.children.indexOf(child);

    if (index !== -1) {
      this.children.splice(index, 1);
      child.domHandle.remove();
    }
  }

  destroy(): void {
    for (const child of this.children) {
      child.destroy();
    }

    IDPool.release(this.id);
    this.domHandle.remove();
  }
}
