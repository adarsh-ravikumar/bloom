We need to represent the AST containing:

- Widget
- Text
- Expression
- Control
- Parts

into a linear set of instructions.

## Text

```
Text {
  value: String,
}
```

## Span

```
Expression {
  span: Span,
  id: ExpressionId,
}
```

## Parts

```
StartParts
  <Text | Expression>*
EndParts
```

## Widget

```
StartWidget { name: String, id: NodeId }
  StartAttributeList
    AttributeName { name: String, span: Span }
    <Parts | Expression | AttributeTrue>
  EndAttributeList

  ...
EndWidget { id: NodeId, parent: NodeId }
```

## Control

### if

```
StartIfBlock { test: Option<ExpressionId>, id: NodeId }
  StartCaseBlock { test: Option<ExpressionId>, id: NodeId }
    ...
  EndCaseBlock { id: NodeId, parent: NodeId }

  ...

EndIfBlock { id: NodeId, parent: NodeId }
```

### each

```
StartEachBlock { iterable: Option<ExpressionId>, id: NodeId }
  ...
EndEachBlock { id: NodeId, parent: NodeId }
```
