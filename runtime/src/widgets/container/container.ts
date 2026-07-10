import { Widget, type GenericWidget } from "@/core/widget";
import { ContainerView } from "./view";

export type ContainerChild = GenericWidget;

export class Container extends Widget<ContainerView, null> {
  constructor() {
    super(ContainerView, null);
  }
}
