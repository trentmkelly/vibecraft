import test from 'node:test'
import assert from 'node:assert/strict'
import {
  evaluateLoginMilestones,
  formatLoginShardReport,
  runMinimalLoginShard
} from './ci_login_shard.mjs'

test('evaluateLoginMilestones passes readiness, login, configuration, spawn, and disconnect checks', () => {
  assert.deepEqual(evaluateLoginMilestones(completeSession()).map(milestone => [milestone.name, milestone.ok]), [
    ['tcpReadiness', true],
    ['login', true],
    ['configurationOrdering', true],
    ['firstSpawn', true],
    ['unexpectedDisconnect', true]
  ])
})

test('evaluateLoginMilestones fails fast milestones with actionable details', () => {
  const milestones = evaluateLoginMilestones({
    error: new Error('Timed out waiting for 127.0.0.1:25565'),
    timeline: [{ name: 'kicked', summary: ['no'] }],
    packetTrace: []
  })
  assert.deepEqual(milestones.map(milestone => milestone.ok), [false, false, false, false, false])
  assert.match(milestones[0].detail, /Timed out/)
  assert.equal(milestones[4].detail, 'kicked before spawn')
})

test('runMinimalLoginShard combines gate and milestone results', async () => {
  const result = await runMinimalLoginShard({
    runGate: async () => ({
      ok: true,
      session: completeSession(),
      smoke: { checks: [] },
      report: 'gate ok'
    })
  })
  assert.equal(result.ok, true)
  assert.match(result.report, /PASS tcpReadiness/)
  assert.match(result.report, /gate ok/)
})

test('runMinimalLoginShard fails when gate reports smoke failures', async () => {
  const result = await runMinimalLoginShard({
    runGate: async () => ({
      ok: false,
      session: completeSession(),
      smoke: { checks: [{ ok: false, name: 'configurationCompletion', message: 'missing configuration' }] },
      report: 'gate failed'
    })
  })
  assert.equal(result.ok, false)
  assert.match(result.report, /gate failed/)
})

test('formatLoginShardReport emits stable pass and fail lines', () => {
  assert.equal(formatLoginShardReport([
    { name: 'tcpReadiness', ok: true, detail: 'port opened' },
    { name: 'firstSpawn', ok: false, detail: 'login,end' }
  ]), [
    'PASS tcpReadiness: port opened',
    'FAIL firstSpawn: login,end'
  ].join('\n'))
})

function completeSession() {
  return {
    timeline: [
      { name: 'login', summary: [] },
      { name: 'spawn', summary: [] }
    ],
    packetTrace: [
      { state: 'configuration', name: 'finish_configuration' }
    ]
  }
}
