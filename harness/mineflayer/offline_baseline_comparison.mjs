import { createOfflineParitySnapshot } from './offline_parity_snapshot.mjs'
import { runObservedOfflineLogin } from './login_session.mjs'
import { assertParityDiff } from './assertions.mjs'

export async function runOfflineBaselineComparison(options = {}) {
  const official = await (options.runOfficial ?? runObservedOfflineLogin)({
    ...options,
    serverKind: 'official',
    port: options.officialPort ?? options.port,
    root: options.officialRoot
  })
  const envelope = acceptedTimelineEnvelope(official)
  const vibecraft = await (options.runVibeCraft ?? runObservedOfflineLogin)({
    ...options,
    serverKind: 'vibecraft',
    port: options.vibecraftPort ?? options.port,
    root: options.vibecraftRoot
  })
  const diff = compareEnvelope(envelope, vibecraft)
  return {
    ok: diff.length === 0,
    official,
    vibecraft,
    envelope,
    diff,
    assertion: assertParityDiff(diff),
    snapshot: createOfflineParitySnapshot({ official, vibecraft, diff })
  }
}

export function acceptedTimelineEnvelope(session) {
  return {
    profile: session.profile,
    eventNames: (session.timeline ?? []).map(event => event.name),
    packetNames: (session.packetTrace ?? []).map(packet => packet.name)
  }
}

export function compareEnvelope(envelope, session) {
  const diff = []
  const eventNames = (session.timeline ?? []).map(event => event.name)
  const packetNames = (session.packetTrace ?? []).map(packet => packet.name)
  if (JSON.stringify(eventNames) !== JSON.stringify(envelope.eventNames)) {
    diff.push({ path: 'events', official: envelope.eventNames, rebuilt: eventNames })
  }
  if (JSON.stringify(packetNames) !== JSON.stringify(envelope.packetNames)) {
    diff.push({ path: 'packets', official: envelope.packetNames, rebuilt: packetNames })
  }
  return diff
}
