import { runRequiredLoginGate } from './login_gate.mjs'
import { runOfflineModeRegression } from './offline_mode_regression.mjs'
import { runPacketFlowSmoke } from './packet_flow_smoke.mjs'
import { summarizeCompatibilityEvidence, createVanillaCompatibilityPlan } from './vanilla_compatibility.mjs'
import { summarizeGameplaySmoke, createGameplaySmokePlan } from './gameplay_smoke.mjs'

export const CORE_WORKFLOWS = [
  'offline-login',
  'packet-flow',
  'offline-regression',
  'vanilla-compatibility',
  'gameplay-smoke',
  'known-parity-regressions'
]

export function evaluateReleaseGate(evidence, options = {}) {
  const knownRegressions = options.knownRegressions ?? evidence.knownRegressions ?? []
  const checks = [
    checkResult('offline-login', evidence.loginGate),
    checkResult('packet-flow', evidence.packetFlow),
    checkResult('offline-regression', evidence.offlineRegression),
    checkResult('vanilla-compatibility', evidence.vanillaCompatibility),
    checkResult('gameplay-smoke', evidence.gameplaySmoke),
    knownRegressions.length === 0
      ? pass('known-parity-regressions', { count: 0 })
      : fail('known-parity-regressions', 'known vanilla parity regressions remain', { knownRegressions })
  ]
  return {
    ok: checks.every(check => check.ok),
    workflows: CORE_WORKFLOWS,
    checks
  }
}

export async function runReleaseGate(options = {}) {
  const loginGate = await runRequiredLoginGate(options)
  const session = loginGate.session
  const packetFlow = session ? runPacketFlowSmoke(session, options) : { ok: false, checks: [] }
  const offlineRegression = await runOfflineModeRegression({
    ...options,
    runLogin: async () => session
  })
  const vanillaCompatibility = session
    ? summarizeCompatibilityEvidence(session, createVanillaCompatibilityPlan(options))
    : { ok: false, workflows: [] }
  const gameplaySmoke = session
    ? summarizeGameplaySmoke(session, createGameplaySmokePlan(options))
    : { ok: false, steps: {} }
  const gate = evaluateReleaseGate({
    loginGate,
    packetFlow,
    offlineRegression,
    vanillaCompatibility,
    gameplaySmoke,
    knownRegressions: options.knownRegressions ?? []
  })
  return {
    ok: gate.ok,
    gate,
    report: formatReleaseGateReport(gate)
  }
}

export function formatReleaseGateReport(gate) {
  return gate.checks.map(check =>
    `${check.ok ? 'PASS' : 'FAIL'} ${check.name}: ${check.message ?? JSON.stringify(check.details)}`
  ).join('\n')
}

function checkResult(name, result) {
  if (result?.ok) return pass(name, summarizeResult(result))
  return fail(name, `${name} failed`, summarizeResult(result))
}

function summarizeResult(result) {
  if (!result) return { present: false }
  if (Array.isArray(result.checks)) {
    return {
      checks: result.checks.map(check => ({ name: check.name, ok: check.ok }))
    }
  }
  if (Array.isArray(result.regression?.checks)) {
    return {
      checks: result.regression.checks.map(check => ({ name: check.name, ok: check.ok }))
    }
  }
  if (Array.isArray(result.workflows)) return { workflows: result.workflows }
  if (result.steps) return { steps: result.steps }
  return { ok: Boolean(result.ok), skipped: Boolean(result.skipped) }
}

function pass(name, details) {
  return { ok: true, name, details }
}

function fail(name, message, details) {
  return { ok: false, name, message, details }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const knownRegressions = (process.env.RUSTCRAFT_KNOWN_PARITY_REGRESSIONS ?? '')
    .split(/\r?\n|,/)
    .map(entry => entry.trim())
    .filter(Boolean)
  const result = await runReleaseGate({
    binary: process.env.RUSTCRAFT_BIN,
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565),
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.RUSTCRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.RUSTCRAFT_KEEP_ARTIFACTS === '1',
    knownRegressions
  })
  if (result.ok) {
    console.log(result.report)
  } else {
    console.error(result.report)
    process.exitCode = 1
  }
}
