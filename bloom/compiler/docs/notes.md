# Compiler Behaviour v0.1

## Root

The root has the following fields:

- fragment -> The markup part of the entire document.

## Fragments

A fragment is a construct that holds a list of nodes.

There are 3 types of nodes:

- Text
- RegularElement
- Widget

The entire document is a fragment.

## Regular Element

  The regular element is any markup element that is not a user defined widget. It has the following fields:

- attributes -> list of attributes
- fragment   -> content

### Tag Handling

  The `<` character acts as the opening of a tag. Encountering `<` anywhere, regardless of context, puts the parser in the "open-or-close-tag" state.
  If the character after `<` is `/`, then the parser enters "close-tag" state. Else, the compiler enters the "open-tag" state.

  The compiler expects the following structure to open a tag:
    `LANGLE IDENT ATTRIBUTE* SPACE? RANGLE`

  and the following to close a tag:
    `LANGLE FWD_SLASH IDENT SPACE? RANGLE`

  If the compiler encounters
    `LANGLE IDENT ATTRIBUTE* SPACE? FWD_SLASH RANGLE`
  
  when in the "open-tag" state, the current tag is considered "self-closing".

  Otherwise, the opened tag's name is pushed onto the open-stack.

  It is expected that when a tag closes, it matches the top of the open-stack. Otherwise, the compiler errors.

  If a tag is left open, i.e the stack contains items at the end of parsing, that is also an error.

  Spaces after `LANGLE` are not tollerated.

  `IDENT` must be of the format `LETTER (LETTER | DIGIT | HYPHEN | UNDERSCORE)*`

### Attributes

Attributes have the following syntax:
  `IDENT (SPACE* EQ SPACE* VALUE)?`

- Upon encountering an identifier, the parser skips ahead of all the spaces to test for an `=`
- If not found, then the identifier is a "boolean attribute" of value true.
- If an = is found, then the value following it is taken as the value.

The value can be one of:

- `QUOTE text QUOTE`
- `text`

If the value is NOT surrounded by quotes, then the all the text until a space is encountered is taken as the value.
`UNQUOTED_VALUE = characters until SPACE or RANGLE`

If the value is surrounded by quotes, then the inner-text entirely is taken as the value

### Parsing content

  All content from the character after the opening tag to the character before the closing tag is considered the fragment of that tag.
  
  Encountering a `<` must constitute the start of a tag.

  This is a recursive process.

## Text

Any content that is not classified as a candidate for a regular element, it is hence consired a text node.
`>` has no special meaning while parsing text.

> For v0.1, every parsed element is a RegularElement. User-defined widgets are outside the scope of this version.
