import type { Subscriber } from "./state";
import { Widget } from "./widget";

export abstract class StatefulWidget extends Widget implements Subscriber<any> {
  notify(delta: any): void {
    this.update();
  }
}
