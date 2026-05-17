export function normalizeLoginArtifacts(value, options = {}) {
  const context = {
    roots: options.roots ?? [],
    ports: (options.ports ?? []).map(String),
    usernames: options.usernames ?? [],
    randomizedUsernamePattern: options.randomizedUsernamePattern ?? /\b[A-Za-z]+Bot[-_][A-Za-z0-9]+\b/g
  }
  return normalizeValue(value, context)
}

export function normalizeLoginDiff(diff, options = {}) {
  return normalizeLoginArtifacts(diff, options)
}

export function normalizeLoginSessionArtifact(session, options = {}) {
  return normalizeLoginArtifacts({
    profile: session.profile,
    uuid: session.uuid,
    timeline: session.timeline,
    packetTrace: session.packetTrace,
    serverLogs: session.serverLogs,
    parityDiff: session.parityDiff
  }, options)
}

function normalizeValue(value, context) {
  if (value == null) return value
  if (typeof value === 'string') return normalizeText(value, context)
  if (typeof value === 'number' || typeof value === 'boolean') return value
  if (Array.isArray(value)) return value.map(item => normalizeValue(item, context))
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, nested]) => [key, normalizeValue(nested, context)])
    )
  }
  return value
}

function normalizeText(text, context) {
  let output = text
  for (const root of context.roots) output = output.replaceAll(root, '<run-dir>')
  for (const port of context.ports) output = output.replaceAll(port, '<port>')
  for (const username of context.usernames) output = output.replaceAll(username, '<username>')
  return output
    .replace(context.randomizedUsernamePattern, '<username>')
    .replace(/\b\d{4}-\d{2}-\d{2}[T ][0-9:.Z+-]+/g, '<timestamp>')
    .replace(/\b\d{13}\b/g, '<timestamp-ms>')
}
