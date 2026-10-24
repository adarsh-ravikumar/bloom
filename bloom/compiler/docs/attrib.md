verify_attrib:
  for node in ast:
    for attrib in node:
      if registry.is_valid(node.name, attrib):
        continue
      else:
        error
