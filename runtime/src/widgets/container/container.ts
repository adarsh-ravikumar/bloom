import { Widget } from "@/core/widget";
import { ContainerView } from "./view";

export class Container extends Widget {
  constructor(...children: Widget[]) {
    super(ContainerView, ...children);

    this.update();
  }

  public build(): ContainerView {
    return this.view!;
  }
}
