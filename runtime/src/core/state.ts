export interface Subscriber<T> {
  notify(delta: T): void;
}

export class State<T> {
  private _value: T;
  private _subscribers: Subscriber<T>[] = [];

  constructor(value: T) {
    this._value = value;
  }

  public get value(): T {
    return this._value;
  }

  public set value(value: T) {
    if (Object.is(value, this._value))
      return;

    this._value = value;

    this._subscribers.forEach((subscriber) => {
      subscriber.notify(this._value);
    })
  }

  public subscribe(subscriber: Subscriber<T>) {
    this._subscribers.push(subscriber);
  }
}

export class Computed<T> extends State<T> {
  private readonly _states: State<any>[] = [];
  private _callback: () => T;

  constructor(states: readonly State<any>[], callback: () => T) {
    super(callback());

    states.forEach((state) => {
      state.subscribe(this);
      this._states.push(state);
    })

    this._callback = callback;
  }

  notify(_: any) {
    this.value = this._callback();
  }
}
