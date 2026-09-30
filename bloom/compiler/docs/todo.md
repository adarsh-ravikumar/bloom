Now that we get a rich enough AST, the next steps would probably be:

- Identify the script tag
- Have oxc parse it and spit out an AST
- Build a symbol table on the global scope of the script tag from the AST
- Evaluate all expressions with oxc and obtain an AST
- Finally, ensure all identifiers used in the expressions are defined and accessible in the script tag
- We don't really care about anything else.

- After the "checking" step, we need to generate the code.
