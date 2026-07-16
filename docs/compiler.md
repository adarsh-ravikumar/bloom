# Bloom Compiler

The Bloom compiler transforms a TypeScript widget library into a language-independent reflection graph.

The reflection graph is the single source of truth for every generated artifact.

The compiler never emits Rust, registries or schemas directly.

Instead, it first constructs the reflection graph.

Every generator consumes the same graph.

```text
TypeScript

      │

      ▼

Reflection Graph

      │

 ┌────┼──────────────┐
 ▼    ▼              ▼

Rust
Registry
Schema
Documentation
Inspector
Serialization
```

---

# Design Philosophy

The compiler is built as a sequence of small passes.

Each pass has exactly one responsibility.

Passes never overlap responsibilities.

This keeps the compiler deterministic, testable and easy to extend.

---

# Pipeline

```text
Filesystem
    │
    ▼
Discovery
    │
    ▼
Parser
    │
    ▼
Indexer
    │
    ▼
Resolve Widgets
    │
    ▼
Build Reflection Graph
```

Every pass consumes the output of the previous pass.

---

# Discovery

The discovery pass identifies every source file that may participate in reflection.

Discovery does not parse TypeScript.

Its only responsibility is locating files.

Output:

```text
DiscoveredSourceFile[]
```

---

# Parser

The parser constructs a single ts-morph project.

The parser is responsible for:

* Loading every source file
* Configuring the TypeScript compiler
* Producing semantic types

Output:

```text
Project
```

---

# Indexer

The indexer walks the project once and records declarations.

The index is purely an acceleration structure.

It avoids repeatedly traversing the project.

Example:

```text
Classes

Button
Container
Workspace

Interfaces

ButtonProps
Color
Theme
```

The index contains no reflection information.

---

# Resolve Widgets

The widget resolver discovers every reflected widget.

A widget is identified by the `@Component` decorator.

Each widget is resolved into its specialization.

Example:

```ts
class Button extends Widget<
    ButtonProps,
    ButtonRenderer,
    ButtonBehaviour
>
```

becomes

```text
Button

Props:
ButtonProps

Renderer:
ButtonRenderer

Behaviour:
ButtonBehaviour
```

The widget resolver produces the roots of the reflection graph.

---

# Build Reflection Graph

The reflection graph is constructed through graph traversal.

Widgets act as graph roots.

The compiler recursively visits every reachable symbol.

Example:

```text
Counter

↓

CounterProps

↓

Theme

↓

Color
```

Only reachable declarations become part of the reflection graph.

Unused declarations are ignored.

This dramatically reduces generated code and compilation time.

---

# Lowering

The compiler separates traversal from translation.

Traversal discovers declarations.

Lowering converts declarations into Bloom IR.

```text
InterfaceDeclaration

↓

ReflectionInterface
```

Each declaration kind has its own lowerer.

```text
lowers/

interface.ts
property.ts
type.ts
primitive.ts
array.ts
reference.ts
```

Every lowerer performs exactly one transformation.

---

# Type Lowering

TypeScript types are normalized into Bloom types.

The compiler does not expose TypeScript directly.

Instead,

```ts
string[]
```

becomes

```text
Array

↓

Primitive(string)
```

and

```ts
Theme
```

becomes

```text
Reference(Theme)
```

The reflection graph is therefore language-independent.

---

# Graph Traversal

The compiler performs recursive traversal over the reflection graph.

Every symbol is visited at most once.

```text
CounterProps

↓

Theme

↓

Color

↓

Color (already visited)

×

ignored
```

Duplicate visits are prevented through a visited symbol table.

---

# Interface Flattening

Interfaces are flattened during lowering.

Example:

```ts
interface WidgetProps {

    visible: boolean;

}

interface ButtonProps extends WidgetProps {

    text: string;

}
```

becomes

```text
ButtonProps

visible

text
```

Generators never need to reason about inheritance.

Property shadowing follows TypeScript semantics.

Derived interfaces replace inherited properties with the same name.

---

# Reflection Graph

The reflection graph is Bloom's intermediate representation.

It is completely independent of TypeScript.

After lowering, the compiler no longer depends on ts-morph.

Every subsequent generator consumes the same graph.

---

# Design Principles

* The compiler is pass-based.
* Every pass has one responsibility.
* Traversal and lowering are independent.
* Reflection is language-independent.
* The reflection graph is the compiler's IR.
* TypeScript is an input language, not the reflection model.
* Every generator consumes the same reflection graph.
* Runtime reflection never exists.
