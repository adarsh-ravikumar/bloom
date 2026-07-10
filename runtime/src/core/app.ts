import { Container } from "@/widgets/container";
import { Registry } from "./registry";
import type { GenericWidget } from "./widget";

export class App {
  public appName: string;
  public widgets: Registry<GenericWidget> = new Registry();

  public root!: Container;

  constructor(appName: string) {
    this.appName = appName;
  }

  public populateWidgetRegistry(...children: GenericWidget[]): void {
    children.forEach((child) => {
      this.widgets.register(child);
      this.populateWidgetRegistry(...child.children);
    })
  }


  public mount(root: Container): void {
    this.root = root;

    this.populateWidgetRegistry(root);

    this.root.mount();

    document.body.appendChild(this.root.dom);

    this.root.update();
  }
}
