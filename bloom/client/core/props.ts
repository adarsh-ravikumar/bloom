import { Computed, State } from "./state";
import type { GenericWidget } from "./widget";

export class Props<T extends object> {
  private readonly _states =
    {} as { [K in keyof T]: State<T[K]> };

  public readonly value: T;

  constructor(widget: GenericWidget, initial: T) {
    this.value = new Proxy({} as T, {
      get: (_target, key) => {
        const state = this._states[key as keyof T];
        if (!state) return undefined;

        Computed.track(state);
        return state.value;
      },

      set: (_target, key, value) => {
        if (!this._states[key as keyof T]) return false;
        this._states[key as keyof T].value = value;
        return true;
      }
    })

    this.assign(widget, initial)
  }


  public assign(widget: GenericWidget, props: Partial<T>) {
    for (const key of Object.keys(props) as (keyof T)[]) {
      const value = props[key];

      if (value == undefined) continue;

      if (!this._states[key]) {
        const state = value instanceof State ? value : new State(value);
        state.subscribe(widget);

        this._states[key] = state;
      } else {
        this._states[key].setState(value);
      }
    }

    for (const state of Object.values(this._states)) {
      if (state instanceof Computed)
        state.recompute();
    }
  }

  public getState<K extends keyof T>(key: K): State<T[K]> {
    return this._states[key];
  }

  public getStates(): [Extract<keyof T, string>, State<T[keyof T]>][] {
    return Object.entries(this._states) as
      [Extract<keyof T, string>, State<T[keyof T]>][];
  }
}
