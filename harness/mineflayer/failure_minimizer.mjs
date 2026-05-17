import { runMinimalLoginShard } from './ci_login_shard.mjs'

export async function minimizeLoginFailure(scenario, options = {}) {
  const minimized = minimizedScenario(scenario)
  const result = await (options.runShard ?? runMinimalLoginShard)({
    ...options,
    scenario: minimized,
    username: minimized.profiles[0]?.username,
    version: minimized.version,
    timeoutMs: minimized.timeoutMs,
    properties: minimized.serverProperties,
    packetCapture: minimized.packetCapture
  })
  return {
    ok: result.ok,
    minimized,
    result,
    report: formatFailureMinimizerReport(minimized, result)
  }
}

export function minimizedScenario(scenario) {
  const profile = (scenario.profiles ?? [scenario.profile ?? { username: 'RustCraftBot' }])[0]
  return {
    name: `${scenario.name ?? 'login'}-minimized`,
    version: scenario.version,
    profiles: [profile],
    serverProperties: { ...(scenario.serverProperties ?? {}) },
    tempWorlds: 1,
    propertyFiles: 1,
    timeoutMs: scenario.timeoutMs ?? 30_000,
    packetCapture: {
      enabled: true,
      states: scenario.packetCapture?.states ?? ['login', 'configuration', 'play'],
      raw: scenario.packetCapture?.raw ?? true
    },
    vanillaComparison: scenario.vanillaComparison ?? { mode: 'optional' }
  }
}

export function formatFailureMinimizerReport(minimized, result) {
  return [
    `scenario: ${minimized.name}`,
    `bot: ${minimized.profiles[0]?.username ?? '<missing>'}`,
    `propertyFiles: ${minimized.propertyFiles}`,
    `tempWorlds: ${minimized.tempWorlds}`,
    `packetCapture: ${minimized.packetCapture.enabled ? 'enabled' : 'disabled'}`,
    `result: ${result.ok ? 'pass' : 'fail'}`
  ].join('\n')
}
