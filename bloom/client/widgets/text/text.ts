import { DEFAULT_WIDGET_PROPS, Widget, type WidgetProps } from "@core/widget";

import { TextRenderer } from "./renderer";
import { Bloom } from "@client/core/decorators";

export interface TextProps extends WidgetProps {
  isInline: boolean;
  value: string;
}

export const DEFAULT_TEXT_PROPS: TextProps = {
  ...DEFAULT_WIDGET_PROPS,
  isInline: false,
  value: ""
}

@Bloom()
export class Text extends Widget<TextProps, TextRenderer, null> {
  constructor(value?: string) {
    super(TextRenderer);
    this.initProps(DEFAULT_TEXT_PROPS as TextProps);

    this.props.value = value || "";
  }

  public override compose(): void {
    return;
  }
}
