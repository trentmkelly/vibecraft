import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

import { evaluateLoginToSpawnGate, LOGIN_TO_SPAWN_CONTRACT } from './login_to_spawn_gate.mjs'

const execFileAsync = promisify(execFile)

test('login-to-spawn contract names the official oracle and reusable gate milestones', () => {
  assert.equal(LOGIN_TO_SPAWN_CONTRACT.name, 'mineflayer-offline-login-to-spawn')
  assert.equal(LOGIN_TO_SPAWN_CONTRACT.comparedAgainst, 'official-server.jar')
  assert.equal(LOGIN_TO_SPAWN_CONTRACT.gate, 'reusable-login-gate')
  assert.deepEqual(LOGIN_TO_SPAWN_CONTRACT.milestones, [
    'loaded-entity',
    'spawn-position',
    'tab-list-profile',
    'first-chunk-visibility'
  ])
})

test('login-to-spawn gate does not pass until entity, spawn, tab-list, and chunk milestones exist', () => {
  assert.equal(evaluateLoginToSpawnGate({ play: [] }).ok, false)
  assert.deepEqual(evaluateLoginToSpawnGate({ play: [] }).missing, [
    'loaded-entity',
    'spawn-position',
    'tab-list-profile',
    'first-chunk-visibility'
  ])

  const almostReady = evaluateLoginToSpawnGate({
    play: [49, 70, 72, 97].map(id => ({ id }))
  })
  assert.equal(almostReady.ok, false)
  assert.deepEqual(almostReady.missing, ['first-chunk-visibility'])

  const ready = evaluateLoginToSpawnGate({
    play: [49, 70, 72, 97, 45].map(id => ({ id }))
  })
  assert.equal(ready.ok, true)
  assert.deepEqual(ready.missing, [])
})

test('login-to-spawn gate reports stalled spawn diagnostics before first chunk visibility', () => {
  const gate = evaluateLoginToSpawnGate({
    play: [49, 70, 72, 97].map(id => ({ id })),
    joinState: {
      entityId: 1,
      dimension: 'minecraft:overworld',
      position: { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 }
    }
  })

  assert.equal(gate.ok, false)
  assert.deepEqual(gate.missing, ['first-chunk-visibility'])
  assert.equal(gate.diagnostics.lastReceivedChunk, null)
  assert.equal(gate.diagnostics.entityId, 1)
  assert.equal(gate.diagnostics.dimension, 'minecraft:overworld')
  assert.deepEqual(gate.diagnostics.position, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
})

test('login-to-spawn gate passes against the live raw 26.1.2 join probe', {
  skip: process.env.RUSTCRAFT_RUN_LIVE_LOGIN_TO_SPAWN_TEST !== '1'
}, async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: 'SpawnGate'
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  const result = JSON.parse(stdout)
  const gate = evaluateLoginToSpawnGate(result)
  assert.equal(gate.ok, true, JSON.stringify(gate))
  assert.equal(gate.diagnostics.entityId, 1)
  assert.equal(gate.diagnostics.dimension, 'minecraft:overworld')
  assert.deepEqual(gate.diagnostics.position, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
})
