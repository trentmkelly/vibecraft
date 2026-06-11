import { buildOfflineLoginDiagnostics } from './offline_login_diagnostics.mjs'
import { runMinimalLoginShard } from './ci_login_shard.mjs'

export async function runOfflinePreflight(options = {}) {
  const shard = await (options.runShard ?? runMinimalLoginShard)(options)
  const diagnostics = buildOfflineLoginDiagnostics(shard.gate?.session ?? shard.session ?? {}, options)
  const failures = shard.milestones?.filter(milestone => !milestone.ok) ?? []
  const ok = shard.ok && failures.length === 0
  return {
    ok,
    shard,
    diagnostics,
    artifacts: {
      report: shard.report,
      milestones: shard.milestones ?? [],
      diagnostics
    },
    reason: ok ? null : failureReason(failures, diagnostics)
  }
}

export function preflightFailsFast(preflight) {
  return {
    ok: preflight.ok,
    actionable: !preflight.ok && Boolean(preflight.diagnostics?.username)
      && Boolean(preflight.diagnostics?.expectedUuid)
      && Boolean(preflight.reason),
    reason: preflight.reason
  }
}

function failureReason(failures, diagnostics) {
  if (failures.length > 0) return failures.map(failure => failure.name).join(',')
  if (diagnostics.error) return diagnostics.error
  return diagnostics.normalizedDisconnectComponent === '<none>' ? null : diagnostics.normalizedDisconnectComponent
}
