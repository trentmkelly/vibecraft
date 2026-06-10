export function flattenCommandTree(tree, prefix = '') {
  const nodes = []
  for (const [name, node] of Object.entries(tree ?? {})) {
    const path = prefix ? `${prefix} ${name}` : name
    nodes.push({
      path,
      kind: node.kind ?? 'literal',
      parser: node.parser,
      redirect: node.redirect,
      permission: node.permission ?? 0,
      executable: node.executable === true
    })
    nodes.push(...flattenCommandTree(node.children ?? {}, path))
  }
  return nodes
}

export function compareCommandTrees(officialTree, vibecraftTree) {
  const official = new Map(flattenCommandTree(officialTree).map(node => [node.path, node]))
  const vibecraft = new Map(flattenCommandTree(vibecraftTree).map(node => [node.path, node]))
  const paths = [...new Set([...official.keys(), ...vibecraft.keys()])].sort()
  const differences = []
  for (const path of paths) {
    const left = official.get(path)
    const right = vibecraft.get(path)
    if (!left) {
      differences.push({ path, kind: 'extra-node' })
      continue
    }
    if (!right) {
      differences.push({ path, kind: 'missing-node' })
      continue
    }
    for (const field of ['kind', 'parser', 'redirect', 'permission', 'executable']) {
      if (left[field] !== right[field]) {
        differences.push({
          path,
          kind: `${field}-mismatch`,
          official: left[field],
          vibecraft: right[field]
        })
      }
    }
  }
  return {
    ok: differences.length === 0,
    nodeCount: paths.length,
    differences
  }
}

export function createCommandTreeParityPlan() {
  return {
    name: 'command-tree-parity',
    comparedAgainst: 'official-server.jar command report',
    surfaces: [
      'root-literals',
      'argument-types',
      'redirects',
      'permission-gates',
      'executable-flags'
    ]
  }
}
