class _IDPool {
  next: number = 0;
  free: number[] = [];

  acquire() {
    if (this.free.length > 0)
      return this.free.shift();
    return this.next++;
  }

  release(id: number) {
    if (id < this.next && !this.free.includes(id))
      this.free.push(id)
  }
}

export const IDPool = new _IDPool();
