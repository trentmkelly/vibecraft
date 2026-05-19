import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  USERCACHE_CORRUPTION_CASES,
  USERCACHE_STEPS,
  createUsercacheCorruptionPlan,
  createUsercachePlan,
  runUsercacheCorruptionScenario,
  runUsercacheScenario,
  summarizeUsercache,
  summarizeUsercacheCorruption
} from './usercache_scenarios.mjs'

const expiresOn = '2026-06-18 00:00:00 +0000'

test('createUsercachePlan covers restart and lookup side effects', () => {
  const plan = createUsercachePlan()
  assert.equal(plan.name, 'mineflayer-usercache')
  assert.deepEqual(plan.steps, USERCACHE_STEPS)
})

test('runUsercacheScenario validates generated cache, restart, expiry format, and lookup side effects', async () => {
  const username = 'CacheBot'
  const cacheEntry = { name: username, uuid: offlineUuid(username), expiresOn }
  const result = await runUsercacheScenario({
    username,
    joinProbe: async () => ({ cacheEntry }),
    restartProbe: async () => ({ cacheEntry }),
    lookupProbe: async () => ({ sideEffectsObserved: true })
  })
  assert.equal(result.summary.ok, true)
  assert.equal(summarizeUsercache(result.evidence, result.plan).steps['cached-name-and-uuid'], true)
})

test('createUsercacheCorruptionPlan covers missing, empty, malformed, stale, and duplicate inputs', () => {
  const plan = createUsercacheCorruptionPlan()
  assert.equal(plan.name, 'mineflayer-usercache-corruption')
  assert.deepEqual(plan.cases, USERCACHE_CORRUPTION_CASES)
})

test('runUsercacheCorruptionScenario validates repair for every corruption case', async () => {
  const result = await runUsercacheCorruptionScenario({
    repairProbe: async corruptionCase => {
      const username = `Cache${corruptionCase}`
      return {
        username,
        cacheEntry: { name: username, uuid: offlineUuid(username), expiresOn },
        staleUuidPresent: false
      }
    }
  })
  assert.equal(result.summary.ok, true)
  assert.deepEqual(Object.keys(summarizeUsercacheCorruption(result.evidence, result.plan).cases), USERCACHE_CORRUPTION_CASES)
})

test('runUsercacheCorruptionScenario fails if stale UUID survives repair', async () => {
  await assert.rejects(() => runUsercacheCorruptionScenario({
    cases: ['stale'],
    repairProbe: async () => ({
      username: 'Cachestale',
      cacheEntry: { name: 'Cachestale', uuid: offlineUuid('Cachestale'), expiresOn },
      staleUuidPresent: true
    })
  }), /stale UUID/)
})
