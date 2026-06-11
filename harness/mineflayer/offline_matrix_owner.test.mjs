import test from 'node:test'
import assert from 'node:assert/strict'
import {
  declareLoginScenarioMatrixOwner,
  lintLoginScenarioMatrixOwners,
  requiredFixtureCoverage
} from './offline_matrix_owner.mjs'

test('declareLoginScenarioMatrixOwner records sorted unique fixture ownership', () => {
  assert.deepEqual(declareLoginScenarioMatrixOwner('login-access', ['banned', 'fresh', 'fresh']), {
    name: 'login-access',
    fixtures: ['banned', 'fresh']
  })
})

test('lintLoginScenarioMatrixOwners requires explicit known profile fixture kinds', () => {
  const result = lintLoginScenarioMatrixOwners([
    declareLoginScenarioMatrixOwner('baseline', ['fresh', 'returning']),
    declareLoginScenarioMatrixOwner('auth', ['op', 'whitelisted', 'banned']),
    declareLoginScenarioMatrixOwner('collision', ['duplicate'])
  ])
  assert.equal(result.ok, true)
  assert.deepEqual(requiredFixtureCoverage([
    declareLoginScenarioMatrixOwner('baseline', ['fresh', 'returning']),
    declareLoginScenarioMatrixOwner('auth', ['op', 'whitelisted', 'banned']),
    declareLoginScenarioMatrixOwner('collision', ['duplicate'])
  ]), {
    fresh: true,
    returning: true,
    duplicate: true,
    op: true,
    whitelisted: true,
    banned: true
  })
})

test('lintLoginScenarioMatrixOwners rejects missing and unknown fixture declarations', () => {
  const result = lintLoginScenarioMatrixOwners([
    { name: 'missing' },
    declareLoginScenarioMatrixOwner('bad', ['fresh', 'spectator'])
  ])
  assert.equal(result.ok, false)
  assert.deepEqual(result.issues.map(issue => issue.message), [
    'scenario must declare at least one login profile fixture',
    'unknown login profile fixture spectator'
  ])
})
