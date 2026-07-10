import { Bounds } from "./bounds";
import type { App } from "./app";
import { Entity } from "./entity";
import type { View } from "./view";

type ViewConstructor = new (...args: any[]) => View<any>;

export abstract class Widget extends Entity {
  public parent!: Widget;
  public children: Widget[] = [];

  protected view: View<any> | undefined;
  private app!: App;

  public bounds: Bounds;

  public visible: boolean = true;
  public zIndex: number = 0;

  constructor(view?: ViewConstructor, ...children: Widget[]) {
    super();

    children.forEach((child) => {
      this.addChild(child);
    })

    this.bounds = new Bounds(0, 0, 0, 0);

    if (view) {
      this.view = new view();
    }
  }

  public addChild(child: Widget) {
    child.parent = this;
    this.children.push(child);
  }

  public update() {
    if (this.view) {
      this.view.synchronize(this);
    }

    this.children.forEach((child) => {
      child.update();
    })
  }

  public get dom(): HTMLElement {
    return this.view!.dom;
  }

  public abstract build(): View<any>;
}

