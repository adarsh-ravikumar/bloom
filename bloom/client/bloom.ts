import { EntityRegistry } from "@runtime/registry/registry";
import type {
  GenericWidget
} from "@core/widget";

import { Container } from "@widgets/container";
import type { Workspace } from "@widgets/workspace";
import { EntityIdProvider } from "@core/entity";

export class App {
  public appName: string = "Bloom";

  public widgets: EntityRegistry<GenericWidget> = new EntityRegistry();
  public widgetConstructors: Map<number, Function> = new Map();

  public workspace!: Workspace;

  private static readonly idProvider = new EntityIdProvider();

  constructor(appName?: string) {
    if (appName) this.appName = appName;
  }

  public static getID() {
    return this.idProvider.generate();
  }

  public populateWidgetRegistry(...children: GenericWidget[]): void {
    for (let i = 0; i < children.length; i++) {
      const child = children[i];

      if (child.constructorID == -1) {
        const id = App.getID();
        child.constructorID = id;

        this.widgetConstructors.set(id, child.constructor);
      }

      this.widgets.register(child);

      this.populateWidgetRegistry(...child.children);
    }
  }

  public mount(workspace: Workspace): void {
    document.head.querySelector("title")!.innerText = this.appName;

    this.workspace = workspace;

    this.populateWidgetRegistry(workspace);

    this.workspace.mount();

    this.workspace.update();
  }
}
