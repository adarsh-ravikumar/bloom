```
<:p>
  Hello World!
</:p>
```

should produce the result:

```
class Test {
  constructor () {
    this.id = $id();

    const w0 = document.createElement("p");
    this.root.push(w0);
  }
}
```

```
<:div>
  <:button>
    Button!
  </button>
  <:p>
    Hello World!
  </:p>
</div>
```

should produce the result:

```
class Test {
  constructor () {
    this.id = $id();

    const w0 = document.createElement("div");
    const w2 = document.createElement("button");
    const
    const w3 = document.createElement("p");
    this.root.push(w0);
  }
}
```
