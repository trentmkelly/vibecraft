import assert from 'node:assert/strict'
import { mkdir, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'

import {
  WORLDGEN_ACCEPTANCE_PHASES,
  compareWorldgenFixtureReports,
  loadWorldgenAcceptanceReport,
  validateWorldgenAcceptanceReport
} from './worldgen_acceptance_gates.mjs'

test('worldgen acceptance phases are ordered and cover the expected progression', () => {
  assert.deepEqual(
    WORLDGEN_ACCEPTANCE_PHASES.map(phase => phase.id),
    [
      'flat_generator',
      'noise_terrain',
      'surfaces_carvers',
      'biome_decoration',
      'structures',
      'full_persistence_lighting'
    ]
  )
  assert.deepEqual(
    WORLDGEN_ACCEPTANCE_PHASES.map(phase => phase.order),
    [1, 2, 3, 4, 5, 6]
  )
  assert(WORLDGEN_ACCEPTANCE_PHASES.every(phase => phase.requires.length > 0))
})

test('acceptance report fails closed when required fixture reports are absent', () => {
  const report = validateWorldgenAcceptanceReport()

  assert.equal(report.format, 'rustcraft-worldgen-acceptance-gates-v1')
  assert.equal(report.completeThrough, 'flat_generator')
  assert.equal(report.phases.find(phase => phase.id === 'noise_terrain').fixtureSourcesPresent, false)
  assert.deepEqual(
    report.phases.find(phase => phase.id === 'structures').fixtureIssues,
    ['missing dimension fixture report', 'missing overworld fixture report']
  )
})

test('acceptance report recognizes valid overworld and dimension fixture summaries', () => {
  const report = validateWorldgenAcceptanceReport({
    overworldReport: fixtureReport({
      dimension: 'overworld',
      palette: ['minecraft:stone', 'minecraft:water', 'minecraft:grass_block'],
      heightmaps: ['WORLD_SURFACE', 'OCEAN_FLOOR', 'MOTION_BLOCKING', 'MOTION_BLOCKING_NO_LEAVES']
    }),
    dimensionReport: {
      results: [
        ...fixtureReport({ dimension: 'the_nether' }).results,
        ...fixtureReport({ dimension: 'the_end' }).results
      ]
    }
  })

  assert.equal(report.completeThrough, 'full_persistence_lighting')
  assert(report.phases.every(phase => phase.fixtureIssues.length === 0))
})

test('acceptance report loader reads fixture reports from disk and allows missing reports by default', async () => {
  const dir = path.join('/tmp', `rustcraft-worldgen-gate-${process.pid}`)
  await rm(dir, { recursive: true, force: true })
  await mkdir(dir, { recursive: true })
  const overworldPath = path.join(dir, 'overworld.json')
  const dimensionPath = path.join(dir, 'dimensions.json')
  await writeFile(overworldPath, JSON.stringify(fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })))

  const partial = await loadWorldgenAcceptanceReport({ overworldPath, dimensionPath })
  assert.equal(partial.overworldPath, overworldPath)
  assert.equal(partial.dimensionPath, dimensionPath)
  assert.equal(partial.completeThrough, 'biome_decoration')

  await writeFile(dimensionPath, JSON.stringify({
    results: [
      ...fixtureReport({ dimension: 'the_nether' }).results,
      ...fixtureReport({ dimension: 'the_end' }).results
    ]
  }))

  const complete = await loadWorldgenAcceptanceReport({ overworldPath, dimensionPath, requireReports: true })
  assert.equal(complete.completeThrough, 'full_persistence_lighting')
})

test('fixture stability comparison uses requested chunk stable projections', () => {
  const left = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:water', 'minecraft:stone']
  })
  const right = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })

  assert.deepEqual(compareWorldgenFixtureReports(left, right), {
    ok: true,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: []
  })

  right.results[0].artifacts[0].requestedChunks[0].payloadSha256 = 'b'.repeat(64)
  assert.deepEqual(compareWorldgenFixtureReports(left, right), {
    ok: false,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: ['chunk overworld:0,0 stable projection differs']
  })
})

function fixtureReport ({
  dimension,
  palette = ['minecraft:stone'],
  heightmaps = ['WORLD_SURFACE', 'OCEAN_FLOOR', 'MOTION_BLOCKING', 'MOTION_BLOCKING_NO_LEAVES']
}) {
  return {
    results: [{
      fixture: {
        chunks: [{ x: 0, z: 0, dimension }]
      },
      artifacts: [{
        requestedChunks: [{
          chunkX: 0,
          chunkZ: 0,
          status: 'minecraft:full',
          sectionCount: 24,
          nonEmptySectionCount: 24,
          heightmaps: Object.fromEntries(heightmaps.map(name => [name, { type: 'long_array', entries: 37 }])),
          structures: {
            startKeys: [],
            referenceKeys: []
          },
          blockPalette: palette,
          biomePalette: ['minecraft:plains'],
          payloadSha256: 'c'.repeat(64),
          sections: [{
            y: 0,
            blockPalette: palette,
            blockStatesData: {
              entries: 256,
              sha256: 'a'.repeat(64)
            },
            biomePalette: ['minecraft:plains'],
            biomeData: {
              entries: 0,
              sha256: null
            }
          }]
        }]
      }]
    }]
  }
}
