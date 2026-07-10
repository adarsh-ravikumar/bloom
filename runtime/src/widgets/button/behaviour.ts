import { Behaviour, type BehaviourHandler } from "@/core/behaviour";

export class ButtonBehaviour extends Behaviour {
  constructor() {
    super()
  }

  public onClick(handler: BehaviourHandler<"click", HTMLButtonElement>): void {
    this.addEvent<"click", HTMLButtonElement>
      (
        "click",
        handler
      )
  }
}
