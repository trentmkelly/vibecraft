import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

import { evaluateLoginToSpawnGate } from './login_to_spawn_gate.mjs'

const execFileAsync = promisify(execFile)

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

test('login-to-spawn gate passes against the live raw 26.1.2 join probe', async () => {
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
})
