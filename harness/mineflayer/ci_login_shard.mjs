import { runRequiredLoginGate } from './login_gate.mjs'

export async function runMinimalLoginShard(options = {}) {
  const gate = await (options.runGate ?? runRequiredLoginGate)(options)
  const milestones = evaluateLoginMilestones(gate.session, gate.smoke)
  return {
    ok: gate.ok && milestones.every(milestone => milestone.ok),
    gate,
    milestones,
    report: formatLoginShardReport(milestones, gate.report)
  }
}

export function evaluateLoginMilestones(session = {}, smoke = { checks: [] }) {
  const timeline = session.timeline ?? []
  const packetTrace = session.packetTrace ?? []
  const errorMessage = session.error?.message ?? ''
  const kickedBeforeSpawn = eventBefore(timeline, 'kicked', 'spawn')
  return [
    milestone('tcpReadiness', !/Timed out waiting for .*:\d+/.test(errorMessage), errorMessage || 'port opened'),
    milestone('login', timeline.some(event => event.name === 'login'), eventNames(timeline)),
    milestone(
      'configurationOrdering',
      packetTrace.some(packet => packet.state === 'configuration' && packet.name === 'finish_configuration'),
      packetNames(packetTrace)
    ),
    milestone('firstSpawn', timeline.some(event => event.name === 'spawn'), eventNames(timeline)),
    milestone('unexpectedDisconnect', !kickedBeforeSpawn, kickedBeforeSpawn ? 'kicked before spawn' : 'none')
  ].map(entry => {
    const smokeFailure = smoke.checks.find(check => !check.ok && check.name.toLowerCase().includes(entry.name.toLowerCase()))
    return smokeFailure ? { ...entry, ok: false, detail: smokeFailure.message } : entry
  })
}

export function formatLoginShardReport(milestones, gateReport = '') {
  const shard = milestones.map(milestone =>
    `${milestone.ok ? 'PASS' : 'FAIL'} ${milestone.name}: ${formatDetail(milestone.detail)}`
  ).join('\n')
  return gateReport ? `${shard}\n\n${gateReport}` : shard
}

function milestone(name, ok, detail) {
  return { name, ok, detail }
}

function eventBefore(events, first, second) {
  const firstIndex = events.findIndex(event => event.name === first)
  if (firstIndex === -1) return false
  const secondIndex = events.findIndex(event => event.name === second)
  return secondIndex === -1 || firstIndex < secondIndex
}

function eventNames(events) {
  return events.map(event => event.name).join(',') || '<none>'
}

function packetNames(packets) {
  return packets.map(packet => `${packet.state}:${packet.name}`).join(',') || '<none>'
}

function formatDetail(detail) {
  return typeof detail === 'string' ? detail : JSON.stringify(detail)
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const result = await runMinimalLoginShard({
    binary: process.env.VIBECRAFT_BIN,
    port: Number(process.env.VIBECRAFT_PORT ?? 25565),
    username: process.env.VIBECRAFT_BOT ?? 'VibeCraftCI',
    version: process.env.MINEFLAYER_VERSION,
    timeoutMs: Number(process.env.VIBECRAFT_TIMEOUT_MS ?? 30_000),
    keepArtifacts: process.env.KEEP_ARTIFACTS === '1'
  })
  if (result.ok) {
    console.log(result.report)
  } else {
    console.error(result.report)
    process.exitCode = 1
  }
}
