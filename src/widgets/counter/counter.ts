import { Computed } from "@core/state";
import { Widget, type WidgetProps } from "@core/widget";

import { ContainerRenderer } from "@widgets/container";
import { Button } from "@widgets/button";
import { Text } from "@widgets/text";
import { Bloom } from "@client/core/decorators";

export interface CounterProps extends WidgetProps {
  count: number;
  square: Computed<number>;
}

@Bloom()
export class Counter extends Widget<CounterProps, ContainerRenderer, null> {
  countText!: Text;
  squareText!: Text;
  button!: Button;

  constructor() {
    super();

    this.compose();

    this.initProps({
      count: 0,
      square: new Computed(() => {
        return this.props.count ** 2;
      })
    });
  }

  public override compose(): void {
    this.countText = new Text();

    this.squareText = new Text();
    this.squareText.props.isInline = false;

    this.button = new Button();
    this.button.compose(this.countText);

    this.button.behaviour?.onClick(() => {
      this.props.count += 1;
    })

    super.compose(this.button, this.squareText);
  }

  public override update(): void {
    this.countText.props.value = `Count: ${this.props.count}`;
    this.squareText.props.value = `Square: ${this.props.square}`;

    super.update();
  }
}
