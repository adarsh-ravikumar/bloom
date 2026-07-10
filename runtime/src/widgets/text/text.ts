import { TextView } from "./view";
import { State } from "@/core/state";
import { StatefulWidget } from "@/core/stateful_widget";

export class Text extends StatefulWidget {
  private readonly state: State<string> = new State("");

  public isInline: boolean = false;

  constructor(value: string) {
    super(TextView);

    this.value = value;

    this.state.subscribe(this);
  }

  public get value(): string {
    return this.state.value;
  }

  public set value(val: string) {
    this.state.value = val;
  }

  public build(): TextView {
    return this.view!;
  }
}
