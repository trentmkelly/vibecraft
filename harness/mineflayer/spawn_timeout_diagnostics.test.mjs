import assert from 'node:assert/strict'
import test from 'node:test'

import {
  diagnoseSpawnTimeout,
  forceSlowInitialChunkEvidence
} from './spawn_timeout_diagnostics.mjs'

test('spawn-timeout diagnostics report last chunk, entity, dimension, position, and missing milestone', () => {
  const report = diagnoseSpawnTimeout(forceSlowInitialChunkEvidence())

  assert.equal(report.ok, false)
  assert.equal(report.reason, 'spawn-timeout')
  assert.deepEqual(report.missing, ['first-chunk-visibility'])
  assert.equal(report.lastReceivedChunk, null)
  assert.equal(report.entityId, 1)
  assert.equal(report.dimension, 'minecraft:overworld')
  assert.deepEqual(report.position, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
  assert.deepEqual(report.playPacketIds.slice(0, 3), [49, 70, 10])
})

test('spawn-timeout diagnostics keep the newest chunk index when chunks were partially visible', () => {
  const evidence = forceSlowInitialChunkEvidence()
  evidence.play.push({ id: 45 }, { id: 45 })

  const report = diagnoseSpawnTimeout(evidence)

  assert.deepEqual(report.missing, [])
  assert.equal(report.lastReceivedChunk, 1)
})
