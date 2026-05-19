import test from 'node:test'
import assert from 'node:assert/strict'
import {
  PARITY_COVERAGE_CATALOG,
  createParityCoveragePlan,
  recordParityCoverage,
  runParityCoverageCatalog,
  summarizeParityCoverage
} from './parity_coverage_catalog.mjs'

for (const kind of Object.keys(PARITY_COVERAGE_CATALOG)) {
  test(`createParityCoveragePlan covers ${kind}`, () => {
    const plan = createParityCoveragePlan(kind)
    assert.equal(plan.name, `parity-coverage-${kind.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`)
    assert.deepEqual(plan.steps, PARITY_COVERAGE_CATALOG[kind])
  })

  test(`runParityCoverageCatalog validates all ${kind} steps`, async () => {
    const result = await runParityCoverageCatalog(kind, {
      probe: async plan => ({
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { artifact: plan.kind }]))
      })
    })
    assert.equal(result.summary.ok, true)
  })
}

test('summarizeParityCoverage fails with missing catalog evidence', () => {
  const plan = createParityCoveragePlan('protocol')
  const evidence = { timeline: [] }
  recordParityCoverage(evidence, plan.steps[0])
  assert.equal(summarizeParityCoverage(evidence, plan).ok, false)
})

test('runParityCoverageCatalog reports missing worldgen step', async () => {
  await assert.rejects(() => runParityCoverageCatalog('worldgen', {
    probe: async plan => ({
      steps: Object.fromEntries(plan.steps.filter(step => step !== 'real-generated-chunk-packets').map(step => [step, true]))
    })
  }), /real-generated-chunk-packets/)
})
