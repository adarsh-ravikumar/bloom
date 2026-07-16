export class BloomError extends Error {
  constructor(message: string) {
    super(message);

    this.name = "BloomError";
  }
}
