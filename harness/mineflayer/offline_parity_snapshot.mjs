import { normalizeLoginDiff, normalizeLoginSessionArtifact } from './artifact_normalizer.mjs'
import { formatParityDiff } from './assertions.mjs'

export function createOfflineParitySnapshot(options) {
  const official = options.official
  const vibecraft = options.vibecraft
  const diff = options.diff ?? []
  return {
    scenario: options.scenario ?? 'offline-mode-happy-path',
    official: sessionSnapshot(official, options),
    vibecraft: sessionSnapshot(vibecraft, options),
    normalizedDiff: normalizeLoginDiff(diff, {
      roots: [official.root, vibecraft.root].filter(Boolean),
      ports: [official.endpoint?.port, vibecraft.endpoint?.port].filter(port => port != null),
      usernames: [official.profile?.username, vibecraft.profile?.username].filter(Boolean)
    }),
    diffText: formatParityDiff(diff)
  }
}

export function sessionSnapshot(session, options = {}) {
  return {
    profile: session.profile,
    uuid: session.uuid,
    rawEvents: session.timeline ?? [],
    packetNames: (session.packetTrace ?? []).map(packet => packet.name),
    serverLogExcerpt: excerptLogs(session.serverLogs ?? [], options.maxLogLines ?? 20),
    normalized: normalizeLoginSessionArtifact(session, {
      roots: [session.root].filter(Boolean),
      ports: [session.endpoint?.port].filter(port => port != null),
      usernames: [session.profile?.username].filter(Boolean)
    })
  }
}

function excerptLogs(logs, maxLogLines) {
  return logs
    .flatMap(entry => entry.text.split(/\r?\n/).filter(Boolean).map(text => ({
      stream: entry.stream,
      text
    })))
    .slice(-maxLogLines)
}
