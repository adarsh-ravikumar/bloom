// widget:
// - children
// - state
// - events
//
//
// we don't really care about the children, because we only need them for setting up the DOM tree
// if there are structural changes, then we will need to handle the children
// the children array is there purely for mounting and what not
// if and each blocks can just "redraw" on update
//
// if:
// - all this does is "enable and disable" elements.
// - we can wrap all widgets inside an if block into a div or something like that
//   and just mount and unmount that particular block.
//
// each:
// - similar thing here. we are just recomputing if the state changes
//
// first steps, we don't really need to care about if and each
// step 1 -> spawn a panel
// step 2 -> render widgets in that panel
// step 3 -> have state changes reflect in the UI
//

class Workspace {
  panels: Panel[] = [];
}

class Panel {
  root: Widget;

  constructor(root: Widget) {
    this.root = root;
  }
}

class Widget {
  children: Widget[] = [];
  dom: HTMLElement;

  constructor() {
    this.dom = document.createElement("div");
  }

  addChild(widget: Widget) {
    this.children.push(widget);
    this.dom.appendChild(widget.dom);
  }
}

class Text extends Widget {
  constructor() {
    super();
    this.dom = document.createElement("p");
    this.dom.classList.add("text");
  }

  setText(text: string) {
    this.dom.innerText = text;
  }
}

class Container extends Widget {
  constructor() {
    super();
    this.dom.classList.add("container");
  }
}

class Row extends Container {
  constructor() {
    super();
    this.dom.classList.add("row");
  }
}

class Column extends Container {
  constructor() {
    super();
    this.dom.classList.add("column");
  }
}

// say we build a new component out of these guys.
class MyWidget extends Widget {
  create() {
    /*
     * <row>
     *  <column>
     *    <row>
     *      <text> A </text>
     *    </row>
     *    <row>
     *      <text> B </text>
     *    </row>
     *  </column>
     *  <column>
     *    <row>
     *      <text> C </text>
     *    </row>
     *    <row>
     *      <text> D </text>
     *    </row>
     *    <row>
     *      <text> E </text>
     *    </row>
     *  </column>
     * </row>
     */

    const cont = new Row();
    const col1 = new Column();
    const col1_row1 = new Row();
    const a = new Text();
    a.setText("A");
    col1_row1.addChild(a);
    col1.addChild(col1_row1);

    const col1_row2 = new Row();
    const b = new Text();
    b.setText("B");
    col1_row2.addChild(b);
    col1.addChild(col1_row2);
    cont.addChild(col1)

    const col2 = new Column();
    const col2_row1 = new Row();
    const c = new Text();
    c.setText("C");
    col2_row1.addChild(c);
    col2.addChild(col2_row1);

    const col2_row2 = new Row();
    const d = new Text();
    d.setText("D");
    col2_row2.addChild(d);
    col2.addChild(col2_row2);
    cont.addChild(col2);

    const col2_row3 = new Row();
    const e = new Text();
    e.setText("E");
    col2_row3.addChild(e);
    col2.addChild(col2_row3);
    cont.addChild(col2);

    this.addChild(cont);
  }
}

const w = new MyWidget();
w.create();

document.body.appendChild(w.dom);
