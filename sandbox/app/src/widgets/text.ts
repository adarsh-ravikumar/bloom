import { Computed, State } from "../state";
import { Widget, type WidgetTemplate } from "../widget";

export class Text implements WidgetTemplate {
  text: State<string>;
  widget: Widget;

  constructor(text: string) {
    this.text = new State(text);
  }

  get domHandle() {
    return this.widget.domHandle;
  }

  render() {
    // there's 3 cases of an update
    // A => value change
    // B => attribute change
    // C => structural change
    //
    // we will first deal with value change
    //
    const element = document.createElement("p");

    // now, we want it such that whenever this text changes
    // the element's node value updates.
    new Computed(() => {
      element.innerText = this.text.value
    })

    this.domHandle.appendChild(element);
  }

}
