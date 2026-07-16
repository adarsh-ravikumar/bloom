import type { Entity, EntityID } from "../core/entity";

export class EntityRegistry<T extends Entity> {
  map: Map<EntityID, T>;

  constructor() {
    this.map = new Map();
  }

  get(id: EntityID): T | undefined {
    return this.map.get(id);
  }

  register(item: T) {
    this.map.set(item.id, item);
    item.register();
  }
}
