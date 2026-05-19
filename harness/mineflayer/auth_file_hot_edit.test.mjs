import test from 'node:test'
import assert from 'node:assert/strict'
import {
  AUTH_FILE_HOT_EDIT_STEPS,
  createAuthFileHotEditPlan,
  recordAuthHotEdit,
  runAuthFileHotEdit,
  summarizeAuthFileHotEdit
} from './auth_file_hot_edit.mjs'
import { offlineUuid } from './runner.mjs'

test('createAuthFileHotEditPlan covers every mutable auth file and reload command', () => {
  const plan = createAuthFileHotEditPlan()
  assert.equal(plan.name, 'mineflayer-auth-file-hot-edit')
  assert.deepEqual(plan.steps, AUTH_FILE_HOT_EDIT_STEPS)
  assert.deepEqual(plan.files, ['ops.json', 'whitelist.json', 'banned-players.json', 'banned-ips.json'])
  assert.deepEqual(plan.reloadCommands, ['reload', 'whitelist reload'])
  assert.equal(plan.expectedUuids.op, offlineUuid(plan.profiles.op))
  assert.equal(plan.serverProperties['enforce-whitelist'], 'true')
})

test('summarizeAuthFileHotEdit requires all auth hot-edit steps', () => {
  const plan = createAuthFileHotEditPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordAuthHotEdit(evidence, step)
  assert.equal(summarizeAuthFileHotEdit(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeAuthFileHotEdit(evidence, plan).ok, false)
})

test('runAuthFileHotEdit validates current and reconnecting bot reload effects', async () => {
  const result = await runAuthFileHotEdit({
    onlineProbe: async plan => ({ joined: [plan.profiles.plain, plan.profiles.op] }),
    opsHotEditProbe: async () => ({ reloaded: true, currentOp: true, reconnectOp: true }),
    whitelistHotEditProbe: async () => ({ reloaded: true, allowed: true, unlistedRejected: true }),
    playerBanHotEditProbe: async () => ({ reloaded: true, reconnectRejected: true, reason: 'multiplayer.disconnect.banned' }),
    ipBanHotEditProbe: async () => ({ reloaded: true, reconnectRejected: true, reason: 'multiplayer.disconnect.ip_banned' }),
    currentBotProbe: async () => ({ observedReload: true, commandTreeChanged: true }),
    reconnectProbe: async () => ({ observedReload: true, rejectedNames: ['HotBanned'] }),
    pardonProbe: async () => ({ reloaded: true, playerBanCleared: true, ipBanCleared: true, whitelistCleared: true })
  })
  assert.equal(result.summary.ok, true)
})

test('runAuthFileHotEdit fails when player ban reload does not reject reconnect', async () => {
  await assert.rejects(() => runAuthFileHotEdit({
    onlineProbe: async plan => ({ joined: [plan.profiles.plain, plan.profiles.op] }),
    opsHotEditProbe: async () => ({ reloaded: true, currentOp: true, reconnectOp: true }),
    whitelistHotEditProbe: async () => ({ reloaded: true, allowed: true, unlistedRejected: true }),
    playerBanHotEditProbe: async () => ({ reloaded: true, reconnectRejected: false }),
    ipBanHotEditProbe: async () => ({ reloaded: true, reconnectRejected: true, reason: 'multiplayer.disconnect.ip_banned' }),
    currentBotProbe: async () => ({ observedReload: true }),
    reconnectProbe: async () => ({ observedReload: true }),
    pardonProbe: async () => ({ reloaded: true, playerBanCleared: true, ipBanCleared: true, whitelistCleared: true })
  }), /banned-players\.json/)
})
