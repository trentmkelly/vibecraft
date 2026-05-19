import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  IDENTITY_NORMALIZATION_STEPS,
  createIdentityNormalizationPlan,
  recordNormalization,
  runIdentityNormalization,
  summarizeIdentityNormalization
} from './identity_normalization.mjs'

test('createIdentityNormalizationPlan covers casing, UUID, storage, selector, and logs', () => {
  const plan = createIdentityNormalizationPlan({ baseName: 'NormCase' })
  assert.equal(plan.name, 'mineflayer-identity-normalization')
  assert.deepEqual(plan.usernames, ['NormCase', 'normcase', 'NORMCASE'])
  assert.deepEqual(plan.steps, IDENTITY_NORMALIZATION_STEPS)
})

test('summarizeIdentityNormalization requires every normalization step', () => {
  const plan = createIdentityNormalizationPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordNormalization(evidence, step)
  assert.equal(summarizeIdentityNormalization(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeIdentityNormalization(evidence, plan).ok, false)
})

test('runIdentityNormalization validates case-sensitive UUID and auxiliary probes', async () => {
  const usernames = ['NormCase', 'normcase', 'NORMCASE']
  const result = await runIdentityNormalization({
    usernames,
    loginProbe: async username => ({
      profile: {
        username,
        actualUuid: offlineUuid(username)
      }
    }),
    storedProfileProbe: async plan => ({
      entries: plan.usernames.map(username => ({ name: username, uuid: offlineUuid(username) }))
    }),
    selectorProbe: async plan => ({
      selectors: Object.fromEntries(plan.usernames.map(username => [username, true]))
    }),
    logProbe: async plan => ({
      lines: plan.usernames.map(username => `${username} joined the game`)
    })
  })
  assert.equal(result.summary.ok, true)
})

test('runIdentityNormalization fails on lowercase UUID reuse', async () => {
  await assert.rejects(() => runIdentityNormalization({
    usernames: ['NormCase'],
    loginProbe: async username => ({
      profile: {
        username,
        actualUuid: offlineUuid(username.toLowerCase())
      }
    }),
    storedProfileProbe: async () => ({}),
    selectorProbe: async () => ({}),
    logProbe: async () => ({})
  }), /case-sensitive offline UUID/)
})
