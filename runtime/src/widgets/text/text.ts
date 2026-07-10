import { TextView } from "./view";
import { State } from "@/core/state";
import { Widget } from "@/core/widget";

export class Text extends Widget<TextView, null> {
  private readonly state: State<string> = new State("");

  public isInline: boolean = false;

  constructor(value?: string) {
    super(TextView);

    this.value = value ? value : "";

    this.dependsOn(this.state);
  }

  public override compose(): void {
    return;
  }

  public get value(): string {
    return this.state.value;
  }

  public set value(val: string) {
    this.state.value = val;
  }
}
