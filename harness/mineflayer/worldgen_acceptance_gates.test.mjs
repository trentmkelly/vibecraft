import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  WORLDGEN_ACCEPTANCE_PHASES,
  compareVibecraftWorldgenReport,
  compareWorldgenFixtureReports,
  generateVibecraftWorldgenReport,
  loadWorldgenAcceptanceReport,
  validateWorldgenAcceptanceReport
} from './worldgen_acceptance_gates.mjs'
import { DIMENSION_FIXTURE_CASES } from './vanilla_worldgen_fixtures.mjs'

const execFileAsync = promisify(execFile)

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

  assert.equal(report.format, 'vibecraft-worldgen-acceptance-gates-v1')
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
      cases: dimensionFixtureCases(),
      results: [
        ...fixtureReport({ dimension: 'the_nether' }).results,
        ...fixtureReport({ dimension: 'the_end' }).results
      ]
    }
  })

  assert.equal(report.completeThrough, 'full_persistence_lighting')
  assert(report.phases.every(phase => phase.fixtureIssues.length === 0))
})

test('acceptance report requires named overworld target fixture categories', () => {
  const report = validateWorldgenAcceptanceReport({
    overworldReport: fixtureReport({
      dimension: 'overworld',
      cases: [{ category: 'ocean_target' }],
      palette: ['minecraft:stone', 'minecraft:water']
    })
  })
  const noiseTerrain = report.phases.find(phase => phase.id === 'noise_terrain')

  assert(noiseTerrain.fixtureIssues.includes('overworld report missing mountain_target fixture'))
  assert(noiseTerrain.fixtureIssues.includes('overworld report missing river_target fixture'))
  assert(noiseTerrain.fixtureIssues.includes('overworld report missing ore_vein_heavy_target fixture'))
})

test('acceptance report requires full nether and end dimension matrix fixtures', () => {
  const report = validateWorldgenAcceptanceReport({
    dimensionReport: {
      cases: [{ id: 'the_nether_parity_matrix_seed_0' }],
      results: [
        ...fixtureReport({ dimension: 'the_nether' }).results,
        ...fixtureReport({ dimension: 'the_end' }).results
      ]
    }
  })
  const structures = report.phases.find(phase => phase.id === 'structures')

  assert(structures.fixtureIssues.includes('dimension report missing the_nether_parity_matrix_seed_1 fixture'))
  assert(structures.fixtureIssues.includes('dimension report missing the_end_parity_matrix_seed_0 fixture'))
  assert(structures.fixtureIssues.includes('dimension report missing the_end_parity_matrix_seed_2147483647 fixture'))
})

test('acceptance report requires requested chunks for both dimension fixtures', () => {
  const endFixture = fixtureReport({ dimension: 'the_end' }).results[0]
  endFixture.artifacts[0].requestedChunks = []
  const report = validateWorldgenAcceptanceReport({
    dimensionReport: {
      cases: dimensionFixtureCases(),
      results: [
        ...fixtureReport({ dimension: 'the_nether' }).results,
        endFixture
      ]
    }
  })
  const structures = report.phases.find(phase => phase.id === 'structures')

  assert(structures.fixtureIssues.includes('dimension report missing requested the_end chunks'))
  assert(!structures.fixtureIssues.includes('dimension report missing requested the_nether chunks'))
})

test('acceptance report loader reads fixture reports from disk and allows missing reports by default', async () => {
  const dir = path.join('/tmp', `vibecraft-worldgen-gate-${process.pid}`)
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
    cases: dimensionFixtureCases(),
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

test('fixture stability comparison rejects ambiguous fixture dimensions', () => {
  const left = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })
  left.results[0].fixture.chunks.push({ x: 0, z: 0, dimension: 'the_nether' })
  const right = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })

  assert.deepEqual(compareWorldgenFixtureReports(left, right), {
    ok: false,
    comparedChunks: 0,
    leftChunks: 1,
    rightChunks: 1,
    issues: ['left fixture has ambiguous dimensions for chunk 0,0']
  })
})

test('fixture stability comparison rejects requested chunks without integer coordinates', () => {
  const left = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })
  delete left.results[0].artifacts[0].requestedChunks[0].chunkX
  const right = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })

  assert.deepEqual(compareWorldgenFixtureReports(left, right), {
    ok: false,
    comparedChunks: 0,
    leftChunks: 0,
    rightChunks: 1,
    issues: ['left requested chunk <missing>,0 missing integer coordinates']
  })

  left.results[0].artifacts[0].requestedChunks[0].chunkX = 0.25
  assert.deepEqual(compareWorldgenFixtureReports(left, right).issues, [
    'left requested chunk 0.25,0 missing integer coordinates'
  ])
})

test('VibeCraft worldgen comparison fails closed against vanilla requested chunks', () => {
  const vanilla = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })
  const matchingRust = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:water', 'minecraft:stone']
  })

  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, matchingRust), {
    ok: true,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: []
  })

  const missingRust = vibecraftReport({
    dimension: 'overworld',
    chunkX: 1,
    chunkZ: 0,
    palette: ['minecraft:stone', 'minecraft:water']
  })
  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, missingRust), {
    ok: false,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: [
      'missing vibecraft chunk overworld:0,0',
      'missing vanilla chunk overworld:1,0'
    ]
  })

  const divergentRust = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:stone']
  })
  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, divergentRust), {
    ok: false,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: ['chunk overworld:0,0 stable projection differs']
  })
})

test('VibeCraft worldgen comparison rejects chunks without explicit dimensions', () => {
  const vanilla = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })
  const missingDimension = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:water', 'minecraft:stone']
  })
  delete missingDimension.chunks[0].dimension

  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, missingDimension), {
    ok: false,
    comparedChunks: 0,
    leftChunks: 1,
    rightChunks: 0,
    issues: ['VibeCraft chunk 0,0 missing dimension']
  })

  missingDimension.chunks[0].dimension = ''
  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, missingDimension).issues, [
    'VibeCraft chunk 0,0 missing dimension'
  ])
})

test('VibeCraft worldgen comparison rejects chunks without integer coordinates', () => {
  const vanilla = fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })
  const missingCoordinate = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:water', 'minecraft:stone']
  })
  delete missingCoordinate.chunks[0].chunkX

  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, missingCoordinate), {
    ok: false,
    comparedChunks: 0,
    leftChunks: 1,
    rightChunks: 0,
    issues: ['VibeCraft chunk <missing>,0 missing integer coordinates']
  })

  missingCoordinate.chunks[0].x = 0.5
  assert.deepEqual(compareVibecraftWorldgenReport(vanilla, missingCoordinate).issues, [
    'VibeCraft chunk 0.5,0 missing integer coordinates'
  ])
})

test('accepted vanilla snapshot catches VibeCraft worldgen drift', async () => {
  const acceptedPath = path.join(
    path.dirname(new URL(import.meta.url).pathname),
    'fixtures',
    'accepted_worldgen_snapshot.json'
  )
  const accepted = JSON.parse(await readFile(acceptedPath, 'utf8'))
  const matchingRust = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:water', 'minecraft:stone']
  })

  assert.equal(compareVibecraftWorldgenReport(accepted, matchingRust).ok, true)

  const divergentRust = vibecraftReport({
    dimension: 'overworld',
    palette: ['minecraft:stone']
  })
  assert.deepEqual(compareVibecraftWorldgenReport(accepted, divergentRust), {
    ok: false,
    comparedChunks: 1,
    leftChunks: 1,
    rightChunks: 1,
    issues: ['chunk overworld:0,0 stable projection differs']
  })
})

test('VibeCraft report generation runs configured command and verifies report JSON', async () => {
  const dir = path.join('/tmp', `vibecraft-worldgen-report-command-${process.pid}`)
  await rm(dir, { recursive: true, force: true })
  await mkdir(dir, { recursive: true })
  const reportPath = path.join(dir, 'worldgen_chunks.json')
  const command = `${JSON.stringify(process.execPath)} -e ${JSON.stringify(`require('node:fs').writeFileSync(${JSON.stringify(reportPath)}, JSON.stringify(${JSON.stringify(vibecraftReport({ dimension: 'overworld' }))}))`)}`

  assert.equal(
    await generateVibecraftWorldgenReport({ command, cwd: dir, reportPath }),
    reportPath
  )
})

test('CLI can generate and compare the VibeCraft report on demand', async () => {
  const dir = path.join('/tmp', `vibecraft-worldgen-gate-cli-${process.pid}`)
  await rm(dir, { recursive: true, force: true })
  await mkdir(dir, { recursive: true })
  const overworldPath = path.join(dir, 'overworld.json')
  const dimensionPath = path.join(dir, 'dimensions.json')
  const vibecraftPath = path.join(dir, 'worldgen_chunks.json')
  await writeFile(overworldPath, JSON.stringify(fixtureReport({
    dimension: 'overworld',
    palette: ['minecraft:stone', 'minecraft:water']
  })))
  await writeFile(dimensionPath, JSON.stringify({
    cases: dimensionFixtureCases(),
    results: [
      ...fixtureReport({ dimension: 'the_nether' }).results,
      ...fixtureReport({ dimension: 'the_end' }).results
    ]
  }))
  const vibecraftFixture = {
    format: 'vibecraft-worldgen-signatures-v1',
    chunks: [
      ...vibecraftReport({ dimension: 'overworld', palette: ['minecraft:water', 'minecraft:stone'] }).chunks,
      ...vibecraftReport({ dimension: 'the_nether' }).chunks,
      ...vibecraftReport({ dimension: 'the_end' }).chunks
    ]
  }
  const command = `${JSON.stringify(process.execPath)} -e ${JSON.stringify(`require('node:fs').writeFileSync(${JSON.stringify(vibecraftPath)}, JSON.stringify(${JSON.stringify(vibecraftFixture)}))`)}`

  const { stdout } = await execFileAsync(process.execPath, ['worldgen_acceptance_gates.mjs'], {
    cwd: path.dirname(new URL(import.meta.url).pathname),
    env: {
      ...process.env,
      VIBECRAFT_OVERWORLD_FIXTURE_REPORT: overworldPath,
      VIBECRAFT_DIMENSION_FIXTURE_REPORT: dimensionPath,
      VIBECRAFT_GENERATE_RUST_WORLDGEN_REPORT: '1',
      VIBECRAFT_RUST_WORLDGEN_REPORT_COMMAND: command,
      VIBECRAFT_RUST_WORLDGEN_REPORT_PATH: vibecraftPath
    }
  })
  const report = JSON.parse(stdout)

  assert.equal(report.vibecraftReportPath, vibecraftPath)
  assert.equal(report.vibecraftComparison.ok, true)
  assert.equal(report.vibecraftComparison.comparedChunks, 3)
})

function fixtureReport ({
  dimension,
  cases,
  palette = ['minecraft:stone'],
  heightmaps = ['WORLD_SURFACE', 'OCEAN_FLOOR', 'MOTION_BLOCKING', 'MOTION_BLOCKING_NO_LEAVES']
}) {
  return {
    cases: cases ?? [
      { category: 'ocean_target' },
      { category: 'mountain_target' },
      { category: 'river_target' },
      { category: 'cave_heavy_target' },
      { category: 'village_adjacent_target' },
      { category: 'structure_adjacent_target' },
      { category: 'ore_vein_heavy_target' }
    ],
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

function dimensionFixtureCases () {
  return DIMENSION_FIXTURE_CASES
    .filter(fixture => fixture.category.endsWith('_multi_seed_coordinate_matrix'))
    .map(fixture => ({ id: fixture.id }))
}

function vibecraftReport ({
  dimension,
  chunkX = 0,
  chunkZ = 0,
  palette = ['minecraft:stone'],
  heightmaps = ['WORLD_SURFACE', 'OCEAN_FLOOR', 'MOTION_BLOCKING', 'MOTION_BLOCKING_NO_LEAVES']
}) {
  return {
    format: 'vibecraft-worldgen-signatures-v1',
    chunks: [{
      dimension,
      chunkX,
      chunkZ,
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
  }
}
