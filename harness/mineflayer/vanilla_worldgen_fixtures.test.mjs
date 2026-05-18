import assert from 'node:assert/strict'
import test from 'node:test'

import {
  DIMENSION_FIXTURE_CASES,
  OVERWORLD_FIXTURE_CASES,
  fixtureManifest
} from './vanilla_worldgen_fixtures.mjs'

test('overworld fixture manifest covers normal terrain parity categories', () => {
  const categories = new Set(OVERWORLD_FIXTURE_CASES.map(fixture => fixture.category))

  assert(categories.has('plains_forest_spawn'))
  assert(categories.has('region_boundary_negative_coords'))
  assert(categories.has('far_noise_sample'))
  assert.equal(OVERWORLD_FIXTURE_CASES.length, 3)
  for (const fixture of OVERWORLD_FIXTURE_CASES) {
    assert.match(fixture.id, /^overworld_/)
    assert.equal(typeof fixture.seed, 'bigint')
    assert(fixture.chunks.length >= 3)
    assert(fixture.notes.length > 20)
    assert(fixture.chunks.every(chunk => (chunk.dimension ?? 'overworld') === 'overworld'))
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
