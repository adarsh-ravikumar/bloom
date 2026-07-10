import type { Entity, EntityID } from "./entity";

export class Registry<T extends Entity> {
  map: Map<EntityID, T>;

  constructor() {
    this.map = new Map();
  }

  get(id: EntityID): T | undefined {
    return this.map.get(id);
  }

  register(item: T) {
    this.map.set(item.id, item);
    item.isRegistered = true;
  }
}
