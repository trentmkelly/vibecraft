import { assertEventOrder, assertNoUnexpectedKick, normalizeKickedMessage } from './assertions.mjs'
import { assertPacketOrder } from './assertions.mjs'

export function runPacketFlowSmoke(session, options = {}) {
  const checks = [
    checkOfflineLogin(session),
    checkConfigurationCompletion(session),
    checkKeepAlive(session),
    checkChat(session, options.chatMessage ?? 'hello'),
    checkCommandExecution(session, options.command ?? '/list'),
    checkDisconnectReason(session, options.expectedDisconnectReason)
  ]
  return {
    ok: checks.every(check => check.ok),
    checks
  }
}

export function checkOfflineLogin(session) {
  const phases = assertEventOrder(session.timeline, ['login', 'spawn'], { name: 'offline login' })
  const kick = assertNoUnexpectedKick(session.timeline)
  const packets = assertPacketOrder(session.timeline, ['success'])
  return combineCheck('offlineLogin', [phases, kick, packets])
}

export function checkConfigurationCompletion(session) {
  return packetCheck('configurationCompletion', session, packet =>
    packet.state === 'configuration' && packet.name === 'finish_configuration'
  )
}

export function checkKeepAlive(session) {
  const inbound = session.packetTrace.some(packet => packet.state === 'play' && packet.name === 'keep_alive')
  const outbound = session.packetTrace.some(packet =>
    packet.state === 'play' &&
    packet.name === 'keep_alive' &&
    (packet.direction === 'outbound' || packet.direction === 'clientbound-response')
  )
  if (!inbound || !outbound) {
    return fail('keepAlive', 'missing keepalive request/response pair', { inbound, outbound })
  }
  return pass('keepAlive', { inbound, outbound })
}

export function checkChat(session, message) {
  const chatEvent = session.timeline.find(event =>
    event.name === 'message' && event.summary?.some(part => String(part).includes(message))
  )
  const chatPacket = session.packetTrace.find(packet =>
    ['chat', 'player_chat', 'system_chat'].includes(packet.name)
  )
  if (!chatEvent && !chatPacket) {
    return fail('chat', 'missing chat event or packet', { message })
  }
  return pass('chat', { message, event: Boolean(chatEvent), packet: chatPacket?.name })
}

export function checkCommandExecution(session, command) {
  const normalized = command.startsWith('/') ? command : `/${command}`
  const commandPacket = session.packetTrace.find(packet =>
    ['chat_command', 'command', 'chat'].includes(packet.name) &&
    (packet.command === normalized.slice(1) || packet.message === normalized || packet.keys?.includes('command'))
  )
  const action = session.timeline.find(event =>
    event.name === 'action' && event.summary?.[0] === 'issueCommand' && event.summary?.[1]?.command === normalized
  )
  if (!commandPacket && !action) {
    return fail('commandExecution', 'missing command action or packet', { command: normalized })
  }
  return pass('commandExecution', { command: normalized, packet: commandPacket?.name, action: Boolean(action) })
}

export function checkDisconnectReason(session, expectedReason) {
  const kicked = session.timeline.find(event => event.name === 'kicked')
  const disconnect = session.packetTrace.find(packet => ['disconnect', 'kick_disconnect'].includes(packet.name))
  if (!expectedReason) {
    return pass('disconnectReason', {
      reason: kicked ? normalizeKickedMessage(kicked.summary?.[0]) : null,
      packet: disconnect?.name ?? null
    })
  }
  const actual = normalizeKickedMessage(kicked?.summary?.[0] ?? disconnect?.reason ?? disconnect?.message)
  if (actual !== normalizeKickedMessage(expectedReason)) {
    return fail('disconnectReason', 'disconnect reason mismatch', {
      expected: normalizeKickedMessage(expectedReason),
      actual
    })
  }
  return pass('disconnectReason', { reason: actual })
}

export function formatPacketFlowSmoke(result) {
  return result.checks.map(check =>
    `${check.ok ? 'PASS' : 'FAIL'} ${check.name}: ${check.message ?? JSON.stringify(check.details)}`
  ).join('\n')
}

function packetCheck(name, session, predicate) {
  const packet = session.packetTrace.find(predicate)
  if (!packet) return fail(name, `missing ${name} packet`, {})
  return pass(name, { packet: packet.name, state: packet.state })
}

function combineCheck(name, results) {
  const failed = results.filter(result => !result.ok)
  if (failed.length > 0) return fail(name, 'one or more login checks failed', { failed })
  return pass(name, { checks: results.length })
}

function pass(name, details) {
  return { ok: true, name, details }
}

function fail(name, message, details) {
  return { ok: false, name, message, details }
}
