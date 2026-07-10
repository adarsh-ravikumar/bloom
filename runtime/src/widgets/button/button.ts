import { Widget } from "@/core/widget";
import { ButtonView } from "./view";
import { Text } from "../text";
import { ButtonBehaviour } from "./behaviour";

export type ButtonChild = Text;

export class Button extends Widget<ButtonView, ButtonBehaviour> {
  constructor() {
    super(ButtonView, ButtonBehaviour);
  }

  public override addChild(child: ButtonChild): void {
    if (child instanceof Text) {
      child.isInline = true;
    }

    super.addChild(child);
  }

}
