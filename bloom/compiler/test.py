f = open("./playground.bloom")

braces = 0
quote = None

for char in f.read():
    if char == "{" and quote is None:
        braces += 1
        print(f"BRACE++: {braces}")

    if char == "}" and quote is None:
        braces -= 1
        print(f"BRACE--: {braces}")

    if char == quote:
        print(f"UNQUOTE: {quote}")
        quote = None

    if char in ("'", '"', '`'):
        quote = char
        print(f"QUOTE: {quote}")
