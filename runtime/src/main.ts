import { App } from "./core/app";
import { StatefulWidget } from "./core/stateful_widget";
import { Button, ButtonView } from "./widgets/button";
import { Container } from "./widgets/container";
import { Text } from "./widgets/text";

const app = new App("Pane");

class Counter extends StatefulWidget {
  count: number = 0;
  text: Text;
  button: Button;

  constructor() {
    super();

    this.text = new Text(`Count: ${this.count}`);
    this.button = new Button(this.text);
  }

  public build(): ButtonView {
    return this.button.build();
  }
}

app.build(
  new Container(
    new Text("Hello, world!"),
    new Counter()
  )
)
