import { evaluateLoginToSpawnGate } from './login_to_spawn_gate.mjs'

export function diagnoseSpawnTimeout (evidence) {
  const gate = evaluateLoginToSpawnGate(evidence)
  return {
    ok: false,
    reason: 'spawn-timeout',
    missing: gate.missing,
    lastReceivedChunk: gate.diagnostics.lastReceivedChunk,
    entityId: gate.diagnostics.entityId,
    dimension: gate.diagnostics.dimension,
    position: gate.diagnostics.position,
    playPacketIds: gate.diagnostics.playPacketIds
  }
}

export function forceSlowInitialChunkEvidence () {
  return {
    play: [49, 70, 10, 64, 105, 133, 103, 104, 18, 96, 113, 72, 43, 97, 94, 95].map(id => ({ id })),
    joinState: {
      entityId: 1,
      dimension: 'minecraft:overworld',
      position: { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 }
    }
  }
}
