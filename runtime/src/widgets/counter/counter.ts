import { Computed, State } from "@/core/state";
import { Container } from "@/widgets/container";
import { Button } from "../button";
import { Text } from "../text";

export class Counter extends Container {
  count: State<number> = new State(0);
  square: Computed<number>;

  count_text!: Text;
  square_text!: Text;
  button!: Button;

  constructor() {
    super();

    this.square = new Computed([this.count], () => {
      return this.count.value ** 2;
    });

    this.dependsOn(this.count, this.square);
    this.compose();
  }

  public override compose(): void {
    this.count_text = new Text();
    this.square_text = new Text();

    this.button = new Button();
    this.button.compose(this.count_text);

    this.button.behaviour?.onClick(() => {
      this.count.value += 1;
    })

    super.compose(this.button, this.square_text);
  }

  public override update(): void {
    this.count_text.value = `Count: ${this.count.value}`;
    this.square_text.value = `Square: ${this.square.value}`;
    super.update();
  }
}
