import { App } from "@runtime/app";

export type EntityID = number;

export class EntityIdProvider {
  private _nextID = 0;
  public generate(): EntityID {
    return this._nextID++ as EntityID;
  }
}

export abstract class Entity {
  protected isRegistered: boolean = false;

  public id: EntityID = App.getID();

  public register() {
    this.isRegistered = true;
  }
}
