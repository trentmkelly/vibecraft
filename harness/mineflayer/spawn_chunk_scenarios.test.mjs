import test from 'node:test'
import assert from 'node:assert/strict'
import {
  SPAWN_CHUNK_SCENARIOS,
  createSpawnChunkScenarioPlan,
  recordSpawnChunk,
  runSpawnChunkScenario,
  summarizeSpawnChunk
} from './spawn_chunk_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const kind of Object.keys(SPAWN_CHUNK_SCENARIOS)) {
  test(`createSpawnChunkScenarioPlan covers ${kind}`, () => {
    const plan = createSpawnChunkScenarioPlan(kind)
    assert.equal(plan.name, `mineflayer-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.deepEqual(plan.steps, SPAWN_CHUNK_SCENARIOS[kind])
  })

  test(`runSpawnChunkScenario validates all ${kind} steps`, async () => {
    const result = await runSpawnChunkScenario(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { comparedToVanilla: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeSpawnChunk fails when observations are missing', () => {
  const plan = createSpawnChunkScenarioPlan('chunkStreaming')
  const evidence = { timeline: [] }
  recordSpawnChunk(evidence, plan.steps[0])
  assert.equal(summarizeSpawnChunk(evidence, plan).ok, false)
})

test('runSpawnChunkScenario reports missing forced chunk evidence', async () => {
  await assert.rejects(() => runSpawnChunkScenario('forcedChunkVisibility', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.slice(0, -1).map(step => [step, true]))
    })
  }), /forced-chunk-remains-available/)
})
