import test from 'node:test'
import assert from 'node:assert/strict'
import {
  bisectCandidates,
  featureFlagsFromEnv,
  firstFailingMilestone,
  formatLoginBisectReport,
  revisionsFromEnv,
  runLoginBisect
} from './login_bisect.mjs'

test('bisectCandidates expands revisions and feature flag matrices', () => {
  assert.deepEqual(bisectCandidates({
    revisions: ['a', 'b'],
    featureFlags: [{ net: 'old' }, { net: 'new', compression: 'off' }]
  }).map(candidate => candidate.label), [
    'a [net=old]',
    'a [compression=off,net=new]',
    'b [net=old]',
    'b [compression=off,net=new]'
  ])
})

test('runLoginBisect stops at the first failing milestone by default', async () => {
  const visited = []
  const result = await runLoginBisect({
    revisions: ['good', 'bad', 'later'],
    runShard: async ({ candidate }) => {
      visited.push(candidate.revision)
      return candidate.revision === 'bad'
        ? shard(false, [{ name: 'configurationOrdering', ok: false }])
        : shard(true)
    }
  })
  assert.deepEqual(visited, ['good', 'bad'])
  assert.equal(result.ok, false)
  assert.equal(result.firstFailure.revision, 'bad')
  assert.equal(result.firstFailure.firstFailingMilestone.name, 'configurationOrdering')
})

test('runLoginBisect can collect all candidates when requested', async () => {
  const result = await runLoginBisect({
    revisions: ['a', 'b'],
    stopOnFirstFailure: false,
    runShard: async ({ candidate }) => shard(candidate.revision === 'a')
  })
  assert.deepEqual(result.results.map(entry => entry.revision), ['a', 'b'])
  assert.equal(result.ok, false)
})

test('firstFailingMilestone and formatLoginBisectReport expose failure summary', () => {
  assert.deepEqual(firstFailingMilestone(shard(false, [
    { name: 'tcpReadiness', ok: true },
    { name: 'firstSpawn', ok: false }
  ])), { name: 'firstSpawn', ok: false })
  const report = formatLoginBisectReport({
    firstFailure: { label: 'bad' },
    results: [
      { label: 'good', ok: true },
      { label: 'bad', ok: false, firstFailingMilestone: { name: 'login' } }
    ]
  })
  assert.equal(report, [
    'PASS good',
    'FAIL bad first failing milestone=login',
    'first failure: bad'
  ].join('\n'))
})

test('environment helpers parse revisions and feature flags', () => {
  assert.deepEqual(revisionsFromEnv({ RUSTCRAFT_BISECT_REVISIONS: 'HEAD~2,HEAD~1\nHEAD' }), [
    'HEAD~2',
    'HEAD~1',
    'HEAD'
  ])
  assert.deepEqual(featureFlagsFromEnv({ RUSTCRAFT_BISECT_FLAGS: 'network=v1;compression=off,network=v2' }), [
    { network: 'v1', compression: 'off' },
    { network: 'v2' }
  ])
})

function shard(ok, milestones = []) {
  return {
    ok,
    milestones,
    report: ok ? 'pass' : 'fail'
  }
}
