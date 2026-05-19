import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  PROFILE_COLLISION_STEPS,
  createProfileCollisionPlan,
  recordCollision,
  runProfileCollision,
  summarizeProfileCollision
} from './profile_collision.mjs'

test('createProfileCollisionPlan covers case variants and ownership', () => {
  const plan = createProfileCollisionPlan()
  assert.equal(plan.name, 'mineflayer-profile-collision')
  assert.deepEqual(plan.usernames, ['CaseProfile', 'caseprofile'])
  assert.deepEqual(plan.steps, PROFILE_COLLISION_STEPS)
})

test('summarizeProfileCollision requires every collision step', () => {
  const plan = createProfileCollisionPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordCollision(evidence, step)
  assert.equal(summarizeProfileCollision(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeProfileCollision(evidence, plan).ok, false)
})

test('runProfileCollision validates distinct case-variant UUIDs and playerdata', async () => {
  const usernames = ['CaseProfile', 'caseprofile']
  const result = await runProfileCollision({
    usernames,
    loginProbe: async username => ({
      profile: { username, actualUuid: offlineUuid(username) }
    }),
    duplicateProbe: async () => ({ replacedSameExactNameOnly: true }),
    playerdataProbe: async plan => ({
      files: plan.usernames.map(username => `${offlineUuid(username)}.dat`)
    })
  })
  assert.equal(result.summary.ok, true)
})

test('runProfileCollision fails on UUID collision', async () => {
  await assert.rejects(() => runProfileCollision({
    usernames: ['CaseProfile', 'caseprofile'],
    loginProbe: async username => ({
      profile: { username, actualUuid: offlineUuid('CaseProfile') }
    }),
    duplicateProbe: async () => ({}),
    playerdataProbe: async () => ({})
  }), /Expected UUID/)
})
