import { DEFAULT_WIDGET_PROPS, Widget, type WidgetProps } from "@core/widget";
import { Text } from "@widgets/text";

import { ButtonRenderer } from "./renderer";
import { ButtonBehaviour } from "./behaviour";
import { Bloom, Command, Event } from "@client/core/decorators";
import type { BehaviourHandler } from "@client/core/behaviour";

export enum ButtonType {
  Primary,
  Secondary
}

export interface ButtonProps extends WidgetProps {
  type: ButtonType,
}

const DEFAULT_BUTTON_PROPS: ButtonProps = { ...DEFAULT_WIDGET_PROPS, type: ButtonType.Primary };

@Bloom()
export class Button extends Widget<ButtonProps, ButtonRenderer, ButtonBehaviour> {
  text: Text;

  constructor() {
    super(ButtonRenderer, ButtonBehaviour);

    this.text = new Text();

    this.compose(this.text);

    this.initProps(DEFAULT_BUTTON_PROPS as ButtonProps);
  }

  @Command()
  public setText(text: string) {
    this.text.props.value = text;
  }

  @Event()
  protected onClick(
    handler: BehaviourHandler<"click", HTMLButtonElement>,
  ) {
    this.behaviour.onClick(handler);
  }

  public override addChild(): void { }
}
