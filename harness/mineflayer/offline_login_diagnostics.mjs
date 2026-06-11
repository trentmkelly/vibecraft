import { normalizeKickedMessage } from './assertions.mjs'
import { normalizeLoginArtifacts } from './artifact_normalizer.mjs'
import { offlineUuid } from './runner.mjs'

export function buildOfflineLoginDiagnostics(session, options = {}) {
  const username = session.profile?.username ?? options.username ?? 'VibeCraftBot'
  const lastPacket = last(session.packetTrace ?? [])
  const lastEvent = last(session.timeline ?? [])
  return {
    username,
    expectedUuid: session.profile?.expectedUuid ?? offlineUuid(username),
    serverPid: session.server?.child?.pid ?? options.serverPid ?? null,
    port: session.endpoint?.port ?? options.port ?? null,
    lastPacket: lastPacket ? packetSummary(lastPacket) : null,
    lastBotEvent: lastEvent ? eventSummary(lastEvent) : null,
    normalizedDisconnectComponent: normalizedDisconnect(lastEvent, session.timeline ?? []),
    error: session.error?.message ?? null
  }
}

export function assertOfflineLoginDiagnosticsContract(diagnostics) {
  const missing = []
  for (const key of ['username', 'expectedUuid', 'serverPid', 'port', 'lastPacket', 'lastBotEvent', 'normalizedDisconnectComponent']) {
    if (diagnostics[key] == null || diagnostics[key] === '') missing.push(key)
  }
  return {
    ok: missing.length === 0,
    missing
  }
}

function normalizedDisconnect(lastEvent, timeline) {
  const event = [...timeline].reverse().find(candidate => candidate.name === 'kicked' || candidate.name === 'end')
    ?? lastEvent
  const raw = event?.name === 'kicked'
    ? event.summary?.[0]
    : event?.summary?.[0] ?? event?.name ?? null
  if (raw == null) return '<none>'
  return normalizeLoginArtifacts(normalizeKickedMessage(raw), {})
}

function packetSummary(packet) {
  return {
    name: packet.name,
    state: packet.state,
    keys: packet.keys ?? []
  }
}

function eventSummary(event) {
  return {
    name: event.name,
    summary: event.summary ?? []
  }
}

function last(values) {
  return values.length === 0 ? null : values[values.length - 1]
}
