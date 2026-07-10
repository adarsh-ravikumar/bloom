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
    this._value = value;
  }

  public subscribe(subscriber: Subscriber<T>) {
    this._subscribers.push(subscriber);
  }
}
