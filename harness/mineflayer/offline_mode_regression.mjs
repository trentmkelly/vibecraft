import { runObservedOfflineLogin } from './login_session.mjs'
import { shouldRunLoginGate, changedFilesFromEnv } from './login_gate.mjs'
import { runPacketFlowSmoke } from './packet_flow_smoke.mjs'

export const OFFLINE_REGRESSION_PATHS = [
  /^src\/network\//,
  /^src\/player_online_auth\.rs$/,
  /^src\/player_profile_key\.rs$/,
  /^src\/player_list\.rs$/,
  /^src\/registry\//,
  /^src\/network\/configuration\.rs$/,
  /^src\/network\/login\.rs$/,
  /^src\/network\/play\.rs$/,
  /^harness\/mineflayer\/(login|packet|golden|offline_mode)/
]

export function shouldRunOfflineModeRegression(changedFiles, patterns = OFFLINE_REGRESSION_PATHS) {
  return shouldRunLoginGate(changedFiles, patterns)
}

export function evaluateOfflineModeRegression(session, options = {}) {
  const smoke = runPacketFlowSmoke(session, options)
  const checks = [
    checkLoginTimeout(session),
    checkUnexpectedKick(session),
    checkRegistryOrder(session, options),
    checkPlayStateProgress(session, options),
    ...smoke.checks
  ]
  return {
    ok: !session.error && checks.every(check => check.ok),
    checks,
    smoke
  }
}

export async function runOfflineModeRegression(options = {}) {
  const changedFiles = options.changedFiles ?? []
  if (changedFiles.length > 0 && !shouldRunOfflineModeRegression(changedFiles)) {
    return {
      ok: true,
      skipped: true,
      reason: 'no protocol or login paths changed',
      changedFiles
    }
  }

  const session = await (options.runLogin ?? runObservedOfflineLogin)({
    binary: options.binary,
    port: options.port ?? 25565,
    username: options.username ?? 'VibeCraftOfflineRegression',
    version: options.version,
    timeoutMs: options.timeoutMs ?? 30_000,
    keepArtifacts: options.keepArtifacts,
    keepAlive: options.keepAlive,
    root: options.root,
    startServer: options.startServer,
    waitForReady: options.waitForReady,
    connectBot: options.connectBot
  })
  const regression = evaluateOfflineModeRegression(session, options)
  return {
    ok: regression.ok,
    skipped: false,
    session,
    regression,
    report: formatOfflineModeRegressionReport(session, regression)
  }
}

export function formatOfflineModeRegressionReport(session, regression) {
  const lines = [
    `profile: ${session.profile.username}`,
    `uuid: ${session.uuid}`,
    `events: ${session.timeline.map(event => event.name).join(',')}`
  ]
  if (session.error) lines.splice(2, 0, `error: ${session.error.message}`)
  lines.push(...regression.checks.map(check =>
    `${check.ok ? 'PASS' : 'FAIL'} ${check.name}: ${check.message ?? JSON.stringify(check.details)}`
  ))
  return lines.join('\n')
}

function checkLoginTimeout(session) {
  if (!session.error) return pass('loginTimeout', { timedOut: false })
  if (/timed out/i.test(session.error.message)) {
    return fail('loginTimeout', 'offline-mode login timed out', { error: session.error.message })
  }
  return pass('loginTimeout', { error: session.error.message })
}

function checkUnexpectedKick(session) {
  const kicks = session.timeline.filter(event => event.name === 'kicked')
  if (kicks.length > 0) return fail('unexpectedKick', 'offline-mode login was kicked', { kicks })
  return pass('unexpectedKick', { kicks: 0 })
}

function checkRegistryOrder(session, options = {}) {
  const observed = session.packetTrace
    .filter(packet => packet.state === 'configuration' && packet.name === 'registry_data')
    .map(packet => registrySignature(packet))
    .filter(Boolean)
  const expected = options.expectedRegistryOrder
  if (expected && expected.join('|') !== observed.join('|')) {
    return fail('registryOrder', 'registry-order drift detected', { expected, observed })
  }
  if (observed.length === 0) return fail('registryOrder', 'missing registry_data packets', {})
  return pass('registryOrder', { observed })
}

function checkPlayStateProgress(session, options = {}) {
  const playPackets = session.packetTrace.filter(packet => packet.state === 'play')
  const required = options.minimumPlayPackets ?? 2
  if (playPackets.length < required) {
    return fail('playStateStall', 'play-state packets stalled after login', {
      observed: playPackets.length,
      required
    })
  }
  return pass('playStateStall', { observed: playPackets.length })
}

function registrySignature(packet) {
  if (packet.registryId) return packet.registryId
  if (packet.registryCodec?.id) return packet.registryCodec.id
  if (packet.keys?.includes('registryCodec')) return 'registryCodec'
  return packet.name
}

function pass(name, details) {
  return { ok: true, name, details }
}

function fail(name, message, details) {
  return { ok: false, name, message, details }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runOfflineModeRegression({
    binary: process.env.VIBECRAFT_BIN,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    username: process.env.VIBECRAFT_BOT ?? 'VibeCraftOfflineRegression',
    version: process.env.MINEFLAYER_VERSION,
    changedFiles: changedFilesFromEnv(),
    keepArtifacts: process.env.KEEP_ARTIFACTS === '1'
  })
  if (result.skipped) {
    console.log(`Mineflayer offline-mode regression skipped: ${result.reason}`)
  } else if (result.ok) {
    console.log(result.report)
  } else {
    console.error(result.report)
    process.exitCode = 1
  }
}
