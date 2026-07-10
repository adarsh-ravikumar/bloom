const ALPHABET = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

export class EntityID {

  static generate(length: number = 5) {
    const bytes = crypto.getRandomValues(new Uint8Array(length));

    let id = "";

    for (let i = 0; i < length; i++) {
      id += ALPHABET[bytes[i] % ALPHABET.length];
    }

    return id;
  }

}

export abstract class Entity {
  public isRegistered: boolean = false;
  public id: EntityID = EntityID.generate();
}
