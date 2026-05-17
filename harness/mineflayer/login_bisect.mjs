import { runMinimalLoginShard } from './ci_login_shard.mjs'

export async function runLoginBisect(options = {}) {
  const candidates = bisectCandidates(options)
  const results = []
  for (const candidate of candidates) {
    const result = await (options.runShard ?? runMinimalLoginShard)({
      ...options,
      candidate,
      featureFlags: candidate.featureFlags
    })
    const summary = summarizeCandidate(candidate, result)
    results.push(summary)
    if (!summary.ok && options.stopOnFirstFailure !== false) break
  }
  return {
    ok: results.every(result => result.ok),
    firstFailure: results.find(result => !result.ok) ?? null,
    results
  }
}

export function bisectCandidates(options = {}) {
  const revisions = options.revisions?.length ? options.revisions : ['current']
  const featureFlags = options.featureFlags?.length ? options.featureFlags : [null]
  const candidates = []
  for (const revision of revisions) {
    for (const flags of featureFlags) {
      candidates.push({
        revision,
        featureFlags: flags ?? {},
        label: candidateLabel(revision, flags ?? {})
      })
    }
  }
  return candidates
}

export function firstFailingMilestone(shardResult) {
  return shardResult.milestones?.find(milestone => !milestone.ok) ?? null
}

export function formatLoginBisectReport(result) {
  const lines = result.results.map(entry => {
    const prefix = entry.ok ? 'PASS' : 'FAIL'
    const suffix = entry.firstFailingMilestone
      ? ` first failing milestone=${entry.firstFailingMilestone.name}`
      : ''
    return `${prefix} ${entry.label}${suffix}`
  })
  if (result.firstFailure) {
    lines.push(`first failure: ${result.firstFailure.label}`)
  }
  return lines.join('\n')
}

export function revisionsFromEnv(env = process.env) {
  return splitList(env.RUSTCRAFT_BISECT_REVISIONS)
}

export function featureFlagsFromEnv(env = process.env) {
  return splitList(env.RUSTCRAFT_BISECT_FLAGS).map(entry =>
    Object.fromEntries(entry.split(';').filter(Boolean).map(pair => {
      const [key, value = '1'] = pair.split('=')
      return [key, value]
    }))
  )
}

function summarizeCandidate(candidate, shardResult) {
  return {
    ...candidate,
    ok: shardResult.ok,
    firstFailingMilestone: firstFailingMilestone(shardResult),
    report: shardResult.report
  }
}

function candidateLabel(revision, flags) {
  const flagText = Object.entries(flags)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => `${key}=${value}`)
    .join(',')
  return flagText ? `${revision} [${flagText}]` : revision
}

function splitList(value) {
  return (value ?? '')
    .split(/\r?\n|,/)
    .map(entry => entry.trim())
    .filter(Boolean)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runLoginBisect({
    revisions: revisionsFromEnv(),
    featureFlags: featureFlagsFromEnv(),
    stopOnFirstFailure: process.env.RUSTCRAFT_BISECT_ALL !== '1'
  })
  const report = formatLoginBisectReport(result)
  if (result.ok) {
    console.log(report)
  } else {
    console.error(report)
    process.exitCode = 1
  }
}
