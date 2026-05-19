import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  BAN_PARDON_STEPS,
  createBanPardonPlan,
  recordBanPardon,
  runBanPardon,
  summarizeBanPardon
} from './ban_pardon.mjs'

test('createBanPardonPlan covers profile/IP bans and pardons', () => {
  const plan = createBanPardonPlan()
  assert.equal(plan.name, 'mineflayer-ban-pardon')
  assert.deepEqual(plan.steps, BAN_PARDON_STEPS)
})

test('summarizeBanPardon requires every ban/pardon step', () => {
  const plan = createBanPardonPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordBanPardon(evidence, step)
  assert.equal(summarizeBanPardon(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeBanPardon(evidence, plan).ok, false)
})

test('runBanPardon validates ban messages and reconnect UUIDs', async () => {
  const username = 'BanPardonBot'
  const result = await runBanPardon({
    username,
    profileBanProbe: async () => ({ rejected: true, reason: 'multiplayer.disconnect.banned' }),
    ipBanProbe: async () => ({ rejected: true, reason: 'multiplayer.disconnect.ip_banned' }),
    profilePardonProbe: async () => ({ ok: true, profile: { actualUuid: offlineUuid(username) } }),
    ipPardonProbe: async () => ({ ok: true, profile: { actualUuid: offlineUuid(username) } })
  })
  assert.equal(result.summary.ok, true)
})

test('runBanPardon fails on non-vanilla ban message', async () => {
  await assert.rejects(() => runBanPardon({
    username: 'BanPardonBot',
    profileBanProbe: async () => ({ rejected: true, reason: 'nope' }),
    ipBanProbe: async () => ({ rejected: true, reason: 'multiplayer.disconnect.ip_banned' }),
    profilePardonProbe: async () => ({ ok: true, profile: { actualUuid: offlineUuid('BanPardonBot') } }),
    ipPardonProbe: async () => ({ ok: true, profile: { actualUuid: offlineUuid('BanPardonBot') } })
  }), /vanilla-compatible profile ban message/)
})
