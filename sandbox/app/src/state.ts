export interface Subscriber<T> {
  notify(delta: T): void;
}

export class State<T> {
  private _value: T;
  private _subscribers: Subscriber<T>[] = [];

  constructor(value: T) {
    this._value = value;
  }

  public setState(value: T | State<T>) {
    this.value = value instanceof State ? value.value : value;
  }

  public get value(): T {
    Computed.track(this);
    return this._value;
  }

  public set value(value: T) {
    if (Object.is(value, this._value))
      return;

    this._value = value;

    const subscribers = [...this._subscribers.values()];

    for (const subscriber of subscribers) {
      subscriber.notify(this._value);
    }
  }

  public subscribe(subscriber: Subscriber<T>) {
    this._subscribers.push(subscriber);
  }

  public unsubscribe(subscriber: Subscriber<T>) {
    const idx = this._subscribers.indexOf(subscriber);
    this._subscribers.splice(idx, 1);
  }
}

export class Computed<T> extends State<T> {
  private _callback: () => T;

  private static _current?: Computed<any>;
  private _states: State<T>[] = [];

  constructor(callback: () => T) {
    super(undefined as T);
    this._callback = callback;
    this.recompute();
  }

  public recompute() {
    for (const state of this._states)
      state.unsubscribe(this);

    this._states = [];

    Computed._current = this;

    const value = this._callback();

    Computed._current = undefined;

    this.value = value;
  }

  notify(_: any) {
    this.recompute();
  }

  dependsOn(state: State<T>) {
    this._states.push(state);
    state.subscribe(this);
  }

  public static track(state: State<any>) {
    if (!Computed._current)
      return;

    if (state == Computed._current)
      return;

    Computed._current.dependsOn(state);
  }
}
