import test from 'node:test'
import assert from 'node:assert/strict'
import { rm } from 'node:fs/promises'
import {
  OFFLINE_BASELINE_PORT,
  createOfflineModeBaselineScenario,
  summarizeOfflineBaselineScenario
} from './offline_baseline.mjs'

test('createOfflineModeBaselineScenario builds a reusable black-box baseline fixture', async () => {
  const baseline = await createOfflineModeBaselineScenario({
    name: 'foundation-baseline',
    botCount: 2
  })
  try {
    assert.equal(baseline.name, 'foundation-baseline')
    assert.equal(baseline.version, '1.21.6')
    assert.equal(baseline.port, OFFLINE_BASELINE_PORT)
    assert.equal(baseline.seed, 8675309)
    assert.equal(baseline.levelName, 'world')
    assert.deepEqual(baseline.profiles.map(profile => profile.username), ['VibeCraftBot', 'VibeCraftBot1'])
    assert.deepEqual(baseline.profiles.map(profile => profile.expectedUuid), [
      '19cb8045-82d7-35ea-b3b4-61cd2fce634f',
      'c9408825-df69-3a15-95f4-e3e257c6443b'
    ])
    assert.equal(baseline.serverProperties['online-mode'], 'false')
    assert.equal(baseline.serverProperties['enforce-secure-profile'], 'false')
    assert.deepEqual(baseline.packetCapture, {
      enabled: true,
      states: ['login', 'configuration', 'play']
    })
    assert.deepEqual(baseline.vanillaComparison, {
      mode: 'required',
      output: 'artifacts/mineflayer/offline-baseline-parity.json'
    })
  } finally {
    await baseline.cleanup()
  }
})

test('summarizeOfflineBaselineScenario preserves deterministic comparison metadata', async () => {
  const baseline = await createOfflineModeBaselineScenario({
    port: 30123,
    seed: 42,
    levelName: 'baseline-world',
    profilePrefix: 'GateBot',
    vanillaComparisonOutput: 'artifacts/custom-baseline.json'
  })
  try {
    assert.deepEqual(summarizeOfflineBaselineScenario(baseline), {
      name: 'offline-mode-baseline',
      version: '1.21.6',
      port: 30123,
      seed: 42,
      levelName: 'baseline-world',
      profiles: [{
        username: 'GateBot',
        expectedUuid: '544dc8c3-4214-3961-972d-05a139a10d9b'
      }],
      vanillaComparison: {
        mode: 'required',
        output: 'artifacts/custom-baseline.json'
      }
    })
  } finally {
    await rm(baseline.root, { recursive: true, force: true })
  }
})
