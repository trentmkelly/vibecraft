import test from 'node:test'
import assert from 'node:assert/strict'
import {
  OPERATOR_PERMISSION_STEPS,
  createOperatorPermissionPlan,
  recordOperator,
  runOperatorPermissions,
  summarizeOperatorPermissions
} from './operator_permissions.mjs'

test('createOperatorPermissionPlan covers denial, op/deop, reload, and completions', () => {
  const plan = createOperatorPermissionPlan()
  assert.equal(plan.name, 'mineflayer-operator-permissions')
  assert.deepEqual(plan.steps, OPERATOR_PERMISSION_STEPS)
})

test('summarizeOperatorPermissions requires every operator permission step', () => {
  const plan = createOperatorPermissionPlan()
  const evidence = { timeline: [] }
  for (const step of plan.steps) recordOperator(evidence, step)
  assert.equal(summarizeOperatorPermissions(evidence, plan).ok, true)
  evidence.timeline.pop()
  assert.equal(summarizeOperatorPermissions(evidence, plan).ok, false)
})

test('runOperatorPermissions validates op permission surface', async () => {
  const result = await runOperatorPermissions({
    nonOpProbe: async () => ({ denied: true, feedback: 'commands.generic.permission' }),
    opVisibilityProbe: async () => ({ visible: true, suggestions: ['/op', '/deop'] }),
    opCommandProbe: async () => ({ ok: true, feedback: 'commands.op.success' }),
    reloadProbe: async () => ({ reloaded: true, opsJson: [{ name: 'OpBot', level: 4 }] }),
    deopCommandProbe: async () => ({ ok: true, feedback: 'commands.deop.success' }),
    tabCompletionProbe: async () => ({ updated: true, before: [], after: ['/op'] })
  })
  assert.equal(result.summary.ok, true)
})

test('runOperatorPermissions fails when deop feedback is not visible', async () => {
  await assert.rejects(() => runOperatorPermissions({
    nonOpProbe: async () => ({ denied: true }),
    opVisibilityProbe: async () => ({ visible: true }),
    opCommandProbe: async () => ({ ok: true, feedback: 'commands.op.success' }),
    reloadProbe: async () => ({ reloaded: true }),
    deopCommandProbe: async () => ({ ok: true, feedback: 'missing' }),
    tabCompletionProbe: async () => ({ updated: true })
  }), /Expected deop feedback/)
})
