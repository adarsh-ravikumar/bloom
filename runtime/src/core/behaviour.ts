import { Entity } from "./entity";
import type { View } from "./view";

export interface EventMap {
  click: MouseEvent,
  dbclick: MouseEvent,
  pointerdown: PointerEvent,
  pointerup: PointerEvent,
  pointermove: PointerEvent,
  keydown: KeyboardEvent,
  keyup: KeyboardEvent,
  focus: FocusEvent,
  blur: FocusEvent
}

export type BehaviourHandler<K extends keyof EventMap, T extends HTMLElement> = (this: T, event: EventMap[K]) => void;

export abstract class Behaviour extends Entity {
  public readonly events: Map<keyof EventMap, EventListener> = new Map();

  constructor() {
    super();
  }

  protected addEvent<K extends keyof EventMap, T extends HTMLElement>(
    type: K, handler: BehaviourHandler<K, T>
  ): void {
    this.events.set(type, handler as EventListener);
  }

  public attach(view: View<HTMLElement>) {
    this.events.forEach((handler, type) => {
      view.dom.addEventListener(type, handler);
    })
  }

}
