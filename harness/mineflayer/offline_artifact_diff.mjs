import { diffArtifacts, normalizeArtifacts } from './runner.mjs'

export function diffOfflineLoginArtifacts(official, vibecraft) {
  return diffArtifacts(
    normalizeSeed(normalizeArtifacts(official.artifacts, {
      root: official.root,
      port: official.port,
      seed: official.seed
    }), official.seed),
    normalizeSeed(normalizeArtifacts(vibecraft.artifacts, {
      root: vibecraft.root,
      port: vibecraft.port,
      seed: vibecraft.seed
    }), vibecraft.seed)
  )
}

export function artifactDiffSummary(diff) {
  return {
    ok: diff.length === 0,
    paths: diff.map(entry => entry.path)
  }
}

function normalizeSeed(value, seed) {
  if (seed == null) return value
  const seedText = String(seed)
  if (typeof value === 'string') return value.replaceAll(seedText, '<seed>')
  if (Array.isArray(value)) return value.map(item => normalizeSeed(item, seed))
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [key, normalizeSeed(nested, seed)])
    )
  }
  return value
}
