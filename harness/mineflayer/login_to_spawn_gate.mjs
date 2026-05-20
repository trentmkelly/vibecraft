export const LOGIN_TO_SPAWN_MILESTONES = [
  'loaded-entity',
  'spawn-position',
  'tab-list-profile',
  'first-chunk-visibility'
]

export const LOGIN_TO_SPAWN_CONTRACT = {
  name: 'mineflayer-offline-login-to-spawn',
  comparedAgainst: 'official-server.jar',
  gate: 'reusable-login-gate',
  milestones: LOGIN_TO_SPAWN_MILESTONES
}

export function evaluateLoginToSpawnGate (evidence) {
  const playPackets = evidence.play ?? []
  const playIds = playPackets.map(packet => packet.id)
  const chunkPackets = playPackets.filter(packet => packet.id === 45)
  const milestones = {
    'loaded-entity': playIds.includes(49),
    'spawn-position': playIds.includes(72) && playIds.includes(97),
    'tab-list-profile': playIds.includes(70),
    'first-chunk-visibility': chunkPackets.length > 0
  }
  const missing = LOGIN_TO_SPAWN_MILESTONES.filter(name => !milestones[name])

  return {
    ok: missing.length === 0,
    milestones,
    missing,
    diagnostics: {
      lastReceivedChunk: chunkPackets.length === 0 ? null : chunkPackets.length - 1,
      entityId: evidence.joinState?.entityId ?? null,
      dimension: evidence.joinState?.dimension ?? null,
      position: evidence.joinState?.position ?? null,
      playPacketIds: playIds
    }
  }
}
