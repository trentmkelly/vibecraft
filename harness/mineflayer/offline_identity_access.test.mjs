import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  OFFLINE_IDENTITY_ACCESS_STEPS,
  createOfflineIdentityAccessPlan,
  recordIdentity,
  runOfflineIdentityAccess,
  summarizeOfflineIdentityAccess
} from './offline_identity_access.mjs'

test('createOfflineIdentityAccessPlan covers identity and access-file gates', () => {
  const plan = createOfflineIdentityAccessPlan()
  assert.equal(plan.name, 'mineflayer-offline-identity-access')
  assert.deepEqual(plan.steps, OFFLINE_IDENTITY_ACCESS_STEPS)
  assert.ok(plan.steps.includes('username-preserved'))
  assert.ok(plan.steps.includes('offline-uuid-derived'))
  assert.ok(plan.steps.includes('whitelist-rejects-unlisted'))
  assert.ok(plan.steps.includes('ban-rejects-profile'))
  assert.ok(plan.steps.includes('operator-lookup-visible'))
})

test('summarizeOfflineIdentityAccess requires every access milestone', () => {
  const plan = createOfflineIdentityAccessPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordIdentity(evidence, step)
  assert.equal(summarizeOfflineIdentityAccess(evidence, plan).ok, true)

  evidence.timeline.pop()
  const summary = summarizeOfflineIdentityAccess(evidence, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps['operator-lookup-visible'], false)
})

test('runOfflineIdentityAccess validates username, UUID, whitelist, ban, and op evidence', async () => {
  const username = 'IdentityBot'
  const result = await runOfflineIdentityAccess({
    username,
    loginProbe: async () => ({
      profile: {
        username,
        actualUuid: offlineUuid(username)
      }
    }),
    whitelistProbe: async () => ({ rejected: true, reason: 'multiplayer.disconnect.not_whitelisted' }),
    banProbe: async () => ({ rejected: true, reason: 'multiplayer.disconnect.banned' }),
    operatorProbe: async () => ({ operatorVisible: true, level: 4 })
  })

  assert.equal(result.summary.ok, true)
})

test('runOfflineIdentityAccess fails on wrong offline UUID', async () => {
  await assert.rejects(() => runOfflineIdentityAccess({
    username: 'IdentityBot',
    loginProbe: async () => ({
      profile: {
        username: 'IdentityBot',
        actualUuid: '00000000-0000-0000-0000-000000000000'
      }
    }),
    whitelistProbe: async () => ({ rejected: true }),
    banProbe: async () => ({ rejected: true }),
    operatorProbe: async () => ({ operatorVisible: true })
  }), /Expected offline UUID/)
})
