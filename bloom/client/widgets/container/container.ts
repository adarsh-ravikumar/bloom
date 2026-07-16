import { DEFAULT_WIDGET_PROPS, Widget, type GenericWidget, type WidgetProps } from "@core/widget";

import { ContainerRenderer } from "./renderer";
import { Bloom } from "@client/core/decorators";

export type ContainerChild = GenericWidget;

export interface ContainerProps extends WidgetProps {

}

export const DEFAULT_CONTAINER_PROPS: ContainerProps = {
  ...DEFAULT_WIDGET_PROPS
};

@Bloom()
export class Container extends Widget<ContainerProps, ContainerRenderer, null> {
  constructor() {
    super(ContainerRenderer, null);
    this.initProps(DEFAULT_CONTAINER_PROPS as ContainerProps);
  }
}
