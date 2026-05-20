import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

import { compareLoginOrderAgainstOfficial, evaluateLoginOrder, summarizeLoginOrder } from './login_order_gate.mjs'

const execFileAsync = promisify(execFile)

test('login order gate rejects play before configuration finish', () => {
  const gate = evaluateLoginOrder({
    login: 2,
    compressionThreshold: -1,
    config: [{ id: 12 }, { id: 7 }, { id: 14 }, { id: 3 }],
    play: [{ id: 49 }]
  })
  assert.equal(gate.ok, true)

  const bad = evaluateLoginOrder({
    login: 2,
    compressionThreshold: -1,
    config: [{ id: 12 }, { id: 14 }, { id: 7 }, { id: 3 }],
    play: [{ id: 49 }]
  })
  assert.equal(bad.ok, false)
  assert.ok(bad.failures.includes('registry-after-known-packs'))
})

test('login order gate compares RustCraft timeline against official server.jar ordering', () => {
  const official = {
    login: 2,
    compressionThreshold: -1,
    config: [{ id: 12 }, { id: 7 }, { id: 13 }, { id: 14 }, { id: 3 }],
    play: [{ id: 49 }]
  }
  const matching = compareLoginOrderAgainstOfficial({
    login: 2,
    compressionThreshold: -1,
    config: [{ id: 12 }, { id: 7 }, { id: 13 }, { id: 14 }, { id: 3 }],
    play: [{ id: 49 }]
  }, official)
  assert.equal(matching.ok, true)

  const mismatch = compareLoginOrderAgainstOfficial({
    login: 2,
    compressionThreshold: null,
    config: [{ id: 12 }, { id: 14 }, { id: 7 }, { id: 3 }],
    play: [{ id: 49 }]
  }, official)
  assert.equal(mismatch.ok, false)
  assert.deepEqual(mismatch.firstMismatch, {
    index: 2,
    actual: 'login/set_compression_absent',
    official: 'login/set_compression',
    actualContext: [
      'handshake',
      'login_start',
      'login/set_compression_absent',
      'login_success',
      'login_acknowledgement',
      'configuration/update_enabled_features'
    ],
    officialContext: [
      'handshake',
      'login_start',
      'login/set_compression',
      'login_success',
      'login_acknowledgement',
      'configuration/update_enabled_features'
    ]
  })
})

test('login order gate captures raw handshake through join-game order against live server', {
  skip: process.env.RUSTCRAFT_RUN_LIVE_LOGIN_ORDER_TEST !== '1'
}, async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: 'LoginOrder',
        RUSTCRAFT_RAW_PROBE_MODE: 'record'
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  const result = JSON.parse(stdout)
  const gate = evaluateLoginOrder(result)
  assert.equal(gate.ok, true, JSON.stringify(gate))
  assert.deepEqual(summarizeLoginOrder(result).slice(0, 5), [
    'handshake',
    'login_start',
    result.compressionThreshold == null ? 'login/set_compression_absent' : 'login/set_compression',
    'login_success',
    'login_acknowledgement',
  ])
  assert.ok(gate.timeline.includes('configuration/select_known_packs'))
  assert.ok(gate.timeline.includes('configuration/finish_configuration'))
  assert.ok(gate.timeline.includes('play/login'))
})
