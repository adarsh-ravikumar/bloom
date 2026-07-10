import { Bounds } from "./bounds";
import type { App } from "./app";
import { Entity } from "./entity";
import type { View } from "./view";
import type { Behaviour } from "./behaviour";
import type { State } from "./state";

// signature defined here may return null, which causes UB
// use cautiously
type ViewConstructor<T extends View<any> | null> = (new (...args: any[]) => T) | null;
type BehaviourConstructor<T extends Behaviour | null> = (new (...args: any[]) => T) | null;

export type GenericWidget = Widget<any, any>;

export abstract class Widget<V extends View<any> | null, B extends Behaviour | null> extends Entity {
  public parent!: GenericWidget;
  public children: GenericWidget[] = [];

  protected view: V | undefined;
  public behaviour: B | undefined;

  public bounds: Bounds;

  public visible: boolean = true;
  public zIndex: number = 0;

  private composed: boolean = false;
  private mounted: boolean = false;

  protected states: State<any>[] = [];

  constructor(view?: ViewConstructor<V>, behaviour?: BehaviourConstructor<B>) {
    super();

    this.bounds = new Bounds(0, 0, 0, 0);

    if (view)
      this.view = new view();

    if (behaviour)
      this.behaviour = new behaviour();
  }

  public dependsOn(...states: State<any>[]) {
    states.forEach((state) => {
      state.subscribe(this);
      this.states.push(state);
    })
  }

  public addChild(child: GenericWidget) {
    child.parent = this;
    this.children.push(child);
  }

  public update() {
    if (!this.mounted) return;

    this.view?.synchronize(this);

    this.children.forEach((child) => {
      child.update();
    })
  }

  public get dom(): HTMLElement {
    return this.view!.dom;
  }

  public notify(_: any): void {
    this.update();
  }

  public compose(...children: GenericWidget[]): void {
    if (this.composed) return;
    this.composed = true;

    children.forEach((child) => {
      this.addChild(child);
    })
  }

  public mount(): void {
    if (this.mounted) return;
    this.mounted = true;

    if (this.behaviour)
      this.behaviour.attach(this.view!);

    this.children.forEach((child) => {
      child.mount();
      this.view?.dom.appendChild(child.view.dom);
    })
  }

  public get classList(): DOMTokenList {
    return this.view?.dom.classList;
  }
}

