import { Container } from "@/widgets/container";
import { Registry } from "./registry";
import type { Widget } from "./widget";

export class App {
  public appName: string;
  public widgets: Registry<Widget> = new Registry();

  public root!: Container;

  constructor(appName: string) {
    this.appName = appName;
  }

  public populateWidgetRegistry(children: Widget[]): void {
    children.forEach((child) => {
      this.widgets.register(child);
      this.populateWidgetRegistry(child.children);
    })
  }


  public build(root: Container): void {
    this.root = root;

    this.populateWidgetRegistry(root.children);
    document.body.appendChild(this.root.build().dom);
  }
}
