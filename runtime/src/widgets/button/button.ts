import { Widget } from "@/core/widget";
import { ButtonView } from "./view";
import { Text } from "../text";

export type ButtonChild = Text;

export class Button extends Widget {
  constructor(...children: ButtonChild[]) {
    super(ButtonView, ...children);
    this.update();
  }

  public override addChild(child: ButtonChild): void {
    if (child instanceof Text) {
      child.isInline = true;
    }

    super.addChild(child);
  }

  public build(): ButtonView {
    return this.view!;
  }
}
