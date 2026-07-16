import { Widget, type GenericWidget, type WidgetProps } from "@client/core/widget";
import { ContainerRenderer, DEFAULT_CONTAINER_PROPS } from "@widgets/container";
import { ThemeManager, Themes } from "@core/theme";
import { Bloom } from "@client/core/decorators";

export type WorkspaceChild = GenericWidget;


export interface WorkspaceProps extends WidgetProps {

}

const DEFAULT_WORKSPACE_PROPS: WorkspaceProps = {
  ...DEFAULT_CONTAINER_PROPS
};

@Bloom()
export class Workspace extends Widget<WorkspaceProps, ContainerRenderer, null> {
  public themeManager: ThemeManager = new ThemeManager(Themes.dark);

  constructor() {
    super();
    this.initProps(DEFAULT_WORKSPACE_PROPS);
  }

  mount() {
    super.mount();
    document.querySelector("#app")!.appendChild(this.renderer!.dom);
  }
}
