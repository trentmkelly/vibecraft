import assert from 'node:assert/strict'
import test from 'node:test'

import {
  DIMENSION_FIXTURE_CASES,
  OVERWORLD_FIXTURE_CASES,
  OVERWORLD_PARITY_MATRIX_CHUNKS,
  OVERWORLD_PARITY_MATRIX_SEEDS,
  fixtureManifest,
  runDimensionFixtureSuite
} from './vanilla_worldgen_fixtures.mjs'

test('overworld fixture manifest covers normal terrain parity categories', () => {
  const categories = new Set(OVERWORLD_FIXTURE_CASES.map(fixture => fixture.category))

  assert(categories.has('plains_forest_spawn'))
  assert(categories.has('region_boundary_negative_coords'))
  assert(categories.has('far_noise_sample'))
  assert(categories.has('explicit_overworld_seed_coordinate_matrix'))
  assert.equal(OVERWORLD_FIXTURE_CASES.length, 7)
  for (const fixture of OVERWORLD_FIXTURE_CASES) {
    assert.match(fixture.id, /^overworld_/)
    assert.equal(typeof fixture.seed, 'bigint')
    assert(fixture.chunks.length >= 3)
    assert(fixture.notes.length > 20)
    assert(fixture.chunks.every(chunk => (chunk.dimension ?? 'overworld') === 'overworld'))
  }
})

test('overworld fixture manifest covers the explicit seed and coordinate matrix', () => {
  const matrixFixtures = OVERWORLD_FIXTURE_CASES.filter(
    fixture => fixture.category === 'explicit_overworld_seed_coordinate_matrix'
  )
  const expectedChunks = OVERWORLD_PARITY_MATRIX_CHUNKS.map(chunk => `${chunk.x},${chunk.z}`)

  assert.deepEqual(
    matrixFixtures.map(fixture => fixture.seed),
    OVERWORLD_PARITY_MATRIX_SEEDS
  )
  assert.deepEqual(expectedChunks, ['0,0', '1,0', '0,1', '16,16'])
  for (const fixture of matrixFixtures) {
    assert.deepEqual(
      fixture.chunks.map(chunk => `${chunk.x},${chunk.z}`),
      expectedChunks
    )
    assert.match(fixture.notes, /heightmap/)
    assert.match(fixture.notes, /biome/)
    assert.match(fixture.notes, /representative block-position/)
  }
})

test('manifest serializes bigint seeds and marks pending non-overworld coverage', () => {
  const manifest = fixtureManifest()

  assert.equal(manifest.format, 'rustcraft-vanilla-worldgen-fixtures-v1')
  assert.equal(manifest.cases.length, OVERWORLD_FIXTURE_CASES.length + DIMENSION_FIXTURE_CASES.length)
  assert(manifest.cases.every(fixture => typeof fixture.seed === 'string'))
  assert(manifest.cases.some(fixture => fixture.chunks.some(chunk => chunk.dimension === 'the_nether')))
  assert(manifest.cases.some(fixture => fixture.chunks.some(chunk => chunk.dimension === 'the_end')))

  const runnable = fixtureManifest({ includePendingDimensions: false })
  assert.equal(runnable.cases.length, OVERWORLD_FIXTURE_CASES.length)
  assert(runnable.cases.every(fixture => fixture.chunks.every(chunk => chunk.dimension === 'overworld')))
})

test('dimension fixture suite can be dependency-injected for deterministic report shape', async () => {
  const report = await runDimensionFixtureSuite({
    cases: [DIMENSION_FIXTURE_CASES[0]],
    root: '/tmp/not-used',
    output: '/tmp/rustcraft-dimension-fixture-test/report.json',
    timeoutMs: 1,
    runOracle: async fixture => ({
      ok: true,
      plan: { chunks: fixture.chunks },
      artifacts: [],
      logTail: ''
    })
  })

  assert.equal(report.format, 'rustcraft-vanilla-worldgen-dimension-fixtures-v1')
  assert.equal(report.results.length, 1)
  assert.equal(report.results[0].ok, true)
  assert.equal(report.results[0].fixture.chunks[0].dimension, 'the_nether')
})
