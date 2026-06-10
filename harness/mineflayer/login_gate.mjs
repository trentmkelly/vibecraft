import path from 'node:path'
import { runObservedOfflineLogin } from './login_session.mjs'
import { formatPacketFlowSmoke, runPacketFlowSmoke } from './packet_flow_smoke.mjs'

export const LOGIN_GATE_PATHS = [
  /^src\/network\//,
  /^src\/configuration/,
  /^src\/player/,
  /^src\/storage\//,
  /^src\/tick/,
  /^src\/main\.rs$/,
  /^harness\/mineflayer\//
]

export function shouldRunLoginGate(changedFiles, patterns = LOGIN_GATE_PATHS) {
  return changedFiles.some(file => patterns.some(pattern => pattern.test(normalizePath(file))))
}

export async function runRequiredLoginGate(options = {}) {
  const changedFiles = options.changedFiles ?? []
  if (changedFiles.length > 0 && !shouldRunLoginGate(changedFiles, options.patterns ?? LOGIN_GATE_PATHS)) {
    return {
      ok: true,
      skipped: true,
      reason: 'no gated paths changed',
      changedFiles
    }
  }
  const session = await (options.runLogin ?? runObservedOfflineLogin)({
    binary: options.binary,
    port: options.port ?? 25565,
    username: options.username ?? 'VibeCraftGate',
    version: options.version,
    timeoutMs: options.timeoutMs ?? 30_000,
    keepArtifacts: options.keepArtifacts,
    keepAlive: options.keepAlive,
    root: options.root,
    startServer: options.startServer,
    waitForReady: options.waitForReady,
    connectBot: options.connectBot
  })
  const smoke = runPacketFlowSmoke(session, {
    chatMessage: options.chatMessage,
    command: options.command ?? '/list',
    expectedDisconnectReason: options.expectedDisconnectReason
  })
  return {
    ok: !session.error && smoke.ok,
    skipped: false,
    session,
    smoke,
    report: formatLoginGateReport(session, smoke)
  }
}

export function formatLoginGateReport(session, smoke) {
  const lines = [
    `profile: ${session.profile.username}`,
    `uuid: ${session.uuid}`,
    `events: ${session.timeline.map(event => event.name).join(',')}`,
    formatPacketFlowSmoke(smoke)
  ]
  if (session.error) lines.splice(2, 0, `error: ${session.error.message}`)
  return lines.join('\n')
}

export function changedFilesFromEnv(env = process.env) {
  return (env.VIBECRAFT_CHANGED_FILES ?? '')
    .split(/\r?\n|,/)
    .map(file => file.trim())
    .filter(Boolean)
}

function normalizePath(file) {
  return file.split(path.sep).join('/')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runRequiredLoginGate({
    binary: process.env.VIBECRAFT_BIN,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    username: process.env.VIBECRAFT_BOT ?? 'VibeCraftGate',
    version: process.env.MINEFLAYER_VERSION,
    changedFiles: changedFilesFromEnv(),
    keepArtifacts: process.env.KEEP_ARTIFACTS === '1'
  })
  if (result.skipped) {
    console.log(`Mineflayer login gate skipped: ${result.reason}`)
  } else if (result.ok) {
    console.log(result.report)
  } else {
    console.error(result.report)
    process.exitCode = 1
  }
}
