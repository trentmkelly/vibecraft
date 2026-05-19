import test from 'node:test'
import assert from 'node:assert/strict'
import { offlineUuid } from './runner.mjs'
import {
  SAME_NAME_REPLACEMENT_STEPS,
  createSameNameReplacementPlan,
  recordReplacement,
  runSameNameReplacement,
  summarizeSameNameReplacement
} from './same_name_replacement.mjs'

test('createSameNameReplacementPlan covers replacement cleanup and ownership', () => {
  const plan = createSameNameReplacementPlan()
  assert.equal(plan.name, 'mineflayer-same-name-replacement')
  assert.deepEqual(plan.steps, SAME_NAME_REPLACEMENT_STEPS)
})

test('summarizeSameNameReplacement requires every replacement step', () => {
  const plan = createSameNameReplacementPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordReplacement(evidence, step)
  assert.equal(summarizeSameNameReplacement(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeSameNameReplacement(evidence, plan).ok, false)
})

test('runSameNameReplacement records kick, tab replacement, and playerdata ownership', async () => {
  const username = 'DuplicateBot'
  let connectCount = 0
  let cleaned = false
  let secondEnded = false
  const result = await runSameNameReplacement({
    username,
    connectBot: async () => {
      connectCount += 1
      return {
        root: '/tmp/world',
        endpoint: { port: 25565 },
        server: { child: {} },
        uuid: offlineUuid(username),
        profile: { username, actualUuid: offlineUuid(username) },
        timeline: connectCount === 1
          ? [{ name: 'spawn', summary: [] }, { name: 'kicked', summary: ['duplicate'] }]
          : [{ name: 'spawn', summary: [] }],
        bot: {
          players: { [username]: {} },
          end: () => { secondEnded = true }
        },
        cleanup: async () => { cleaned = true }
      }
    },
    waitForSpawn: async () => {}
  })

  assert.equal(result.summary.ok, true)
  assert.equal(connectCount, 2)
  assert.equal(secondEnded, true)
  assert.equal(cleaned, true)
})

test('runSameNameReplacement fails when replacement UUID is not the offline UUID', async () => {
  let connectCount = 0
  await assert.rejects(() => runSameNameReplacement({
    username: 'DuplicateBot',
    connectBot: async () => {
      connectCount += 1
      return {
        uuid: connectCount === 1 ? offlineUuid('DuplicateBot') : 'bad-uuid',
        profile: { username: 'DuplicateBot', actualUuid: 'bad-uuid' },
        timeline: connectCount === 1
          ? [{ name: 'spawn', summary: [] }, { name: 'kicked', summary: ['duplicate'] }]
          : [{ name: 'spawn', summary: [] }],
        bot: { players: { DuplicateBot: {} }, end: () => {} },
        cleanup: async () => {}
      }
    },
    waitForSpawn: async () => {}
  }), /Expected replacement UUID/)
})
