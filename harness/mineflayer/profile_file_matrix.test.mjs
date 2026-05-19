import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  PROFILE_FILE_MATRIX_STEPS,
  createProfileFileMatrixPlan,
  recordProfileFile,
  runProfileFileMatrix,
  summarizeProfileFileMatrix
} from './profile_file_matrix.mjs'

test('createProfileFileMatrixPlan covers fresh, returning, case, ban, whitelist, and op profiles', () => {
  const plan = createProfileFileMatrixPlan()
  assert.equal(plan.name, 'mineflayer-profile-file-matrix')
  assert.deepEqual(plan.steps, PROFILE_FILE_MATRIX_STEPS)
  assert.equal(plan.profiles.fresh, 'ProfileFresh')
  assert.equal(plan.profiles.renamedCase, 'profilefresh')
})

test('summarizeProfileFileMatrix requires every matrix step', () => {
  const plan = createProfileFileMatrixPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordProfileFile(evidence, step)
  assert.equal(summarizeProfileFileMatrix(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeProfileFileMatrix(evidence, plan).ok, false)
})

test('runProfileFileMatrix validates offline UUID/name semantics across profile files', async () => {
  const plan = createProfileFileMatrixPlan()
  const result = await runProfileFileMatrix({
    joinProfile: async username => ({
      profile: { username, actualUuid: offlineUuid(username) }
    }),
    accessProbe: async (username, kind) => ({
      allowed: kind !== 'banned',
      rejected: kind === 'banned',
      operator: kind === 'op',
      username,
      uuid: offlineUuid(username)
    }),
    profileFileProbe: async () => ({
      usercache: [
        plan.profiles.fresh,
        plan.profiles.renamedCase,
        plan.profiles.whitelisted,
        plan.profiles.op
      ].map(name => ({ name, uuid: offlineUuid(name), expiresOn: '2026-06-18 00:00:00 +0000' })),
      playerdata: [
        plan.profiles.fresh,
        plan.profiles.renamedCase,
        plan.profiles.whitelisted,
        plan.profiles.op
      ].map(name => `${offlineUuid(name)}.dat`),
      ops: [{ name: plan.profiles.op, uuid: offlineUuid(plan.profiles.op), level: 4 }],
      whitelist: [{ name: plan.profiles.whitelisted, uuid: offlineUuid(plan.profiles.whitelisted) }],
      bannedPlayers: [{ name: plan.profiles.banned, uuid: offlineUuid(plan.profiles.banned) }]
    })
  })
  assert.equal(result.summary.ok, true)
})

test('runProfileFileMatrix fails when ops.json uses the wrong UUID', async () => {
  const plan = createProfileFileMatrixPlan()
  await assert.rejects(() => runProfileFileMatrix({
    joinProfile: async username => ({
      profile: { username, actualUuid: offlineUuid(username) }
    }),
    accessProbe: async (_username, kind) => ({
      allowed: kind !== 'banned',
      rejected: kind === 'banned',
      operator: kind === 'op'
    }),
    profileFileProbe: async () => ({
      usercache: [
        plan.profiles.fresh,
        plan.profiles.renamedCase,
        plan.profiles.whitelisted,
        plan.profiles.op
      ].map(name => ({ name, uuid: offlineUuid(name) })),
      playerdata: [
        plan.profiles.fresh,
        plan.profiles.renamedCase,
        plan.profiles.whitelisted,
        plan.profiles.op
      ].map(name => `${offlineUuid(name)}.dat`),
      ops: [{ name: plan.profiles.op, uuid: 'bad-uuid', level: 4 }],
      whitelist: [{ name: plan.profiles.whitelisted, uuid: offlineUuid(plan.profiles.whitelisted) }],
      bannedPlayers: [{ name: plan.profiles.banned, uuid: offlineUuid(plan.profiles.banned) }]
    })
  }), /Missing ops\.json offline UUID entry/)
})
