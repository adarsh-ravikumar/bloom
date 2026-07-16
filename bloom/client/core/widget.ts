import { Bounds } from "@core/bounds"; import { Entity } from "@core/entity";
import type { Renderer } from "@core/renderer";
import type { Behaviour } from "@core/behaviour";
import { Props } from "./props";

// signature defined here may return null, which causes UB
// use cautiously
type RendererConstructor<T extends Renderer<any, GenericWidget> | null> = (new (...args: any[]) => T) | null;
type BehaviourConstructor<T extends Behaviour | null> = (new (...args: any[]) => T) | null;

export type GenericWidget = Widget<any, any, any>;

export interface WidgetProps {
  hidden: boolean;
  zIndex: number;
  tooltip?: string;
}

export const DEFAULT_WIDGET_PROPS: WidgetProps = { hidden: false, zIndex: 0 };

export abstract class
  Widget
  <
    P extends WidgetProps,
    V extends Renderer<any, any> | null,
    B extends Behaviour | null
  >
  extends Entity {

  private static _constructorID: number = -1;

  public parent!: GenericWidget;
  public children: GenericWidget[] = [];

  protected renderer: V | undefined;
  public behaviour: B | undefined;

  public bounds: Bounds;

  private _composed: boolean = false;
  private _mounted: boolean = false;
  private _props: Props<P> = new Props(this, DEFAULT_WIDGET_PROPS as P);

  constructor(renderer?: RendererConstructor<V>, behaviour?: BehaviourConstructor<B>) {
    super();

    this.bounds = new Bounds(0, 0, 0, 0);

    if (renderer)
      this.renderer = new renderer(this);

    if (behaviour)
      this.behaviour = new behaviour();
  }

  public addChild(child: GenericWidget) {
    child.parent = this;
    this.children.push(child);
  }

  public compose(...children: GenericWidget[]): void {
    if (this._composed) return;
    this._composed = true;

    children.forEach((child) => {
      this.addChild(child);
    })
  }

  public mount(): void {
    if (this._mounted) return;
    this._mounted = true;

    if (this.behaviour)
      this.behaviour.attach(this.renderer!);

    this.children.forEach((child) => {
      child.mount();
      this.renderer?.dom.appendChild(child.renderer.dom);
    })
  }

  public update() {
    if (!this._mounted) return;

    this.renderer?.synchronize();

    this.children.forEach((child) => {
      child.update();
    })
  }

  public notify(_: any): void {
    this.update();
  }

  public get props(): P {
    return this._props.value;
  }

  protected initProps(props: Partial<P>) {
    this._props.assign(this, props);
  }

  public addClass(token: string) {
    this.renderer!.dom.classList.add(token);
  }

  public toggleClass(token: string) {
    this.renderer!.dom.classList.add(token);
  }

  public removeClass(token: string) {
    this.renderer!.dom.classList.remove(token);
  }

  public get constructorID(): number {
    return (this.constructor as typeof Widget)._constructorID;
  }

  public set constructorID(id: number) {
    if (this.constructorID == -1) {
      (this.constructor as typeof Widget)._constructorID = id;
    }
  }

  public get html(): string {
    return this.renderer!.dom.outerHTML;
  }
}


