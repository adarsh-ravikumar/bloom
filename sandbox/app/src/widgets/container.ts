import { Widget } from "../widget";

export enum Direction {
  Column, Row
};

export class Container extends Widget {
  direction: Direction;
  children: Widget[] = [];

  constructor(direction: Direction) {
    super();
    this.direction = direction;
  }

  public addChild(child: Widget) {
    this.children.push(child);
    this.update();
  }

  public override update(): void {
    let innerHTML = `<div class="container">`

    for (let i = 0; i < this.children.length; i++) {
      this.children[i].update();
      innerHTML += this.children[i].domHandle.outerHTML;
    }

    innerHTML += "</div>";

    this.domHandle.innerHTML = innerHTML;
  }
}
