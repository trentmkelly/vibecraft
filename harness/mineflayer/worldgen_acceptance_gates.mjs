import { readFile } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  DIMENSION_FIXTURE_CASES,
  OVERWORLD_TARGET_FIXTURE_CHUNKS
} from './vanilla_worldgen_fixtures.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

export const WORLDGEN_ACCEPTANCE_PHASES = [
  {
    id: 'flat_generator',
    order: 1,
    requires: [
      'Rust flat chunks serialize concrete block sections',
      'WORLD_SURFACE_WG and OCEAN_FLOOR_WG heightmaps are present',
      'single-biome flat sections encode minecraft:plains'
    ],
    rustTests: [
      'worldgen::tests::flat_generator_materializes_chunk_sections_and_heightmaps'
    ],
    fixtureSources: []
  },
  {
    id: 'noise_terrain',
    order: 2,
    requires: [
      'vanilla requested chunks are status minecraft:full',
      'requested chunks expose 24 saved sections',
      'requested chunks expose per-section block-state data hashes',
      'requested chunks include WORLD_SURFACE, OCEAN_FLOOR, MOTION_BLOCKING, and MOTION_BLOCKING_NO_LEAVES heightmaps'
    ],
    rustTests: [
      'worldgen_comparison::tests::worldgen_chunk_comparison_matrix_is_deterministic_across_many_seeds_and_coordinates'
    ],
    fixtureSources: ['overworld']
  },
  {
    id: 'surfaces_carvers',
    order: 3,
    requires: [
      'requested chunk palettes include vanilla stone/deepslate/surface/fluid blocks',
      'statusCounts include minecraft:carvers neighbors in oracle region reports',
      'per-section block-state data hashes are available for distribution diffs'
    ],
    rustTests: [
      'worldgen_comparison::tests::worldgen_chunk_comparison_fingerprints_change_with_seed_or_coordinate'
    ],
    fixtureSources: ['overworld']
  },
  {
    id: 'biome_decoration',
    order: 4,
    requires: [
      'requested chunk palettes include generated vegetation or feature blocks when vanilla generated them',
      'requested chunks expose biome palettes',
      'fixture suite includes multiple seeds and far coordinates'
    ],
    rustTests: [],
    fixtureSources: ['overworld']
  },
  {
    id: 'structures',
    order: 5,
    requires: [
      'requested chunks expose structure start/reference keys',
      'dimension fixture reports cover nether and end storage paths',
      'overworld fixture reports include structure-adjacent references when vanilla generated them'
    ],
    rustTests: [],
    fixtureSources: ['overworld', 'dimensions']
  },
  {
    id: 'full_persistence_lighting',
    order: 6,
    requires: [
      'region artifact SHA-256 and payload SHA-256 are recorded',
      'heightmap key/length inventory is recorded',
      'section, biome, structure, and payload summaries are stable across official server runs'
    ],
    rustTests: [],
    fixtureSources: ['overworld', 'dimensions']
  }
]

export function validateWorldgenAcceptanceReport ({ overworldReport, dimensionReport } = {}) {
  const availableSources = new Set()
  if (overworldReport) availableSources.add('overworld')
  if (dimensionReport) availableSources.add('dimensions')

  const phases = WORLDGEN_ACCEPTANCE_PHASES.map(phase => ({
    ...phase,
    fixtureSourcesPresent: phase.fixtureSources.every(source => availableSources.has(source)),
    satisfiedByFixtures: phase.fixtureSources.length === 0 || phase.fixtureSources.every(source => availableSources.has(source)),
    fixtureIssues: fixtureIssuesForPhase(phase, { overworldReport, dimensionReport })
  }))
  return {
    format: 'vibecraft-worldgen-acceptance-gates-v1',
    phases,
    completeThrough: completeThrough(phases)
  }
}

export async function loadWorldgenAcceptanceReport ({
  overworldPath = process.env.VIBECRAFT_OVERWORLD_FIXTURE_REPORT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures.json'),
  dimensionPath = process.env.VIBECRAFT_DIMENSION_FIXTURE_REPORT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-dimension-fixtures.json'),
  requireReports = false
} = {}) {
  const [overworldReport, dimensionReport] = await Promise.all([
    readOptionalJson(overworldPath, requireReports),
    readOptionalJson(dimensionPath, requireReports)
  ])
  return {
    overworldPath,
    dimensionPath,
    ...validateWorldgenAcceptanceReport({ overworldReport, dimensionReport })
  }
}

export function compareWorldgenFixtureReports (left, right) {
  const leftComparable = comparableRequestedChunks(left, 'left')
  const rightComparable = comparableRequestedChunks(right, 'right')
  const issues = [...leftComparable.issues, ...rightComparable.issues]
  if (issues.length > 0) {
    return {
      ok: false,
      comparedChunks: 0,
      leftChunks: leftComparable.chunks.length,
      rightChunks: rightComparable.chunks.length,
      issues
    }
  }
  return compareComparableChunkLists(leftComparable.chunks, rightComparable.chunks, { leftLabel: 'left', rightLabel: 'right' })
}

export function compareVibecraftWorldgenReport (vanillaReports, vibecraftReport) {
  const reports = Array.isArray(vanillaReports) ? vanillaReports : [vanillaReports]
  const vanilla = combineComparableResults(reports.map(report => comparableRequestedChunks(report, 'vanilla')))
  if (vanilla.issues.length > 0) {
    return {
      ok: false,
      comparedChunks: 0,
      leftChunks: vanilla.chunks.length,
      rightChunks: 0,
      issues: vanilla.issues
    }
  }
  const vanillaChunks = vanilla.chunks
  const vibecraft = comparableVibecraftChunks(vibecraftReport)
  if (vibecraft.issues.length > 0) {
    return {
      ok: false,
      comparedChunks: 0,
      leftChunks: vanillaChunks.length,
      rightChunks: vibecraft.chunks.length,
      issues: vibecraft.issues
    }
  }
  const vibecraftChunks = vibecraft.chunks
  return compareComparableChunkLists(vanillaChunks, vibecraftChunks, {
    leftLabel: 'vanilla',
    rightLabel: 'vibecraft'
  })
}

export async function generateVibecraftWorldgenReport ({
  command = process.env.VIBECRAFT_RUST_WORLDGEN_REPORT_COMMAND ?? 'cargo run -q -- --report',
  cwd = repoRoot,
  reportPath = process.env.VIBECRAFT_RUST_WORLDGEN_REPORT_PATH ?? path.join(repoRoot, 'generated', 'reports', 'worldgen_chunks.json')
} = {}) {
  await runShellCommand(command, cwd)
  await readOptionalJson(reportPath, true)
  return reportPath
}

function compareComparableChunkLists (leftChunks, rightChunks, { leftLabel, rightLabel }) {
  const leftMap = new Map(leftChunks.map(chunk => [chunk.key, chunk]))
  const rightMap = new Map(rightChunks.map(chunk => [chunk.key, chunk]))
  const issues = []

  for (const key of [...leftMap.keys()].sort()) {
    if (!rightMap.has(key)) {
      issues.push(`missing ${rightLabel} chunk ${key}`)
      continue
    }
    const leftChunk = leftMap.get(key)
    const rightChunk = rightMap.get(key)
    const leftStable = stableChunkProjection(leftChunk)
    const rightStable = stableChunkProjection(rightChunk)
    if (JSON.stringify(leftStable) !== JSON.stringify(rightStable)) {
      issues.push(`chunk ${key} stable projection differs`)
    }
  }

  for (const key of [...rightMap.keys()].sort()) {
    if (!leftMap.has(key)) issues.push(`missing ${leftLabel} chunk ${key}`)
  }

  return {
    ok: issues.length === 0,
    comparedChunks: Math.min(leftMap.size, rightMap.size),
    leftChunks: leftMap.size,
    rightChunks: rightMap.size,
    issues
  }
}

function completeThrough (phases) {
  let complete = null
  for (const phase of [...phases].sort((left, right) => left.order - right.order)) {
    if (!phase.satisfiedByFixtures || phase.fixtureIssues.length > 0) break
    complete = phase.id
  }
  return complete
}

function fixtureIssuesForPhase (phase, { overworldReport, dimensionReport }) {
  const issues = []
  if (phase.fixtureSources.includes('overworld')) {
    issues.push(...validateOverworldReport(overworldReport))
  }
  if (phase.fixtureSources.includes('dimensions')) {
    issues.push(...validateDimensionReport(dimensionReport))
  }
  return [...new Set(issues)].sort()
}

function validateOverworldReport (report) {
  if (!report) return ['missing overworld fixture report']
  const chunks = requestedChunks(report)
  const requiredTargetCategories = OVERWORLD_TARGET_FIXTURE_CHUNKS.map(fixture => fixture.category)
  const reportCategories = new Set((report.cases ?? []).map(fixture => fixture.category))
  const issues = []
  if (chunks.length === 0) issues.push('overworld report has no requested chunks')
  for (const category of requiredTargetCategories) {
    if (!reportCategories.has(category)) issues.push(`overworld report missing ${category} fixture`)
  }
  if (!chunks.every(chunk => chunk.status === 'minecraft:full')) issues.push('not every overworld requested chunk is full')
  if (!chunks.every(chunk => chunk.sectionCount >= 1)) issues.push('overworld requested chunks are missing sections')
  if (!chunks.every(chunk => Object.keys(chunk.heightmaps ?? {}).includes('WORLD_SURFACE'))) issues.push('overworld requested chunks missing WORLD_SURFACE heightmap')
  if (!chunks.every(chunk => Object.keys(chunk.heightmaps ?? {}).includes('OCEAN_FLOOR'))) issues.push('overworld requested chunks missing OCEAN_FLOOR heightmap')
  if (!chunks.every(chunk => Object.keys(chunk.heightmaps ?? {}).includes('MOTION_BLOCKING'))) issues.push('overworld requested chunks missing MOTION_BLOCKING heightmap')
  if (!chunks.every(chunk => Object.keys(chunk.heightmaps ?? {}).includes('MOTION_BLOCKING_NO_LEAVES'))) issues.push('overworld requested chunks missing MOTION_BLOCKING_NO_LEAVES heightmap')
  if (!chunks.some(chunk => (chunk.blockPalette ?? []).includes('minecraft:stone'))) issues.push('overworld requested chunks missing stone palette sample')
  if (!chunks.some(chunk => (chunk.blockPalette ?? []).includes('minecraft:water'))) issues.push('overworld requested chunks missing water palette sample')
  if (!chunks.every(chunk => (chunk.sections ?? []).some(section => section.blockStatesData?.sha256))) issues.push('overworld requested chunks missing section data hashes')
  return issues
}

function validateDimensionReport (report) {
  if (!report) return ['missing dimension fixture report']
  const dimensions = new Set((report.results ?? []).flatMap(result => result.fixture?.chunks?.map(chunk => chunk.dimension) ?? []))
  const chunks = requestedChunks(report)
  const requestedDimensionCounts = requestedDimensionCountsByFixture(report)
  const requiredMatrixIds = DIMENSION_FIXTURE_CASES
    .filter(fixture => fixture.category.endsWith('_multi_seed_coordinate_matrix'))
    .map(fixture => fixture.id)
  const reportCaseIds = new Set((report.cases ?? []).map(fixture => fixture.id))
  const issues = []
  if (!dimensions.has('the_nether')) issues.push('dimension report missing the_nether fixture')
  if (!dimensions.has('the_end')) issues.push('dimension report missing the_end fixture')
  if ((requestedDimensionCounts.get('the_nether') ?? 0) === 0) issues.push('dimension report missing requested the_nether chunks')
  if ((requestedDimensionCounts.get('the_end') ?? 0) === 0) issues.push('dimension report missing requested the_end chunks')
  for (const id of requiredMatrixIds) {
    if (!reportCaseIds.has(id)) issues.push(`dimension report missing ${id} fixture`)
  }
  if (chunks.length === 0) issues.push('dimension report has no requested chunks')
  if (!chunks.every(chunk => chunk.status === 'minecraft:full')) issues.push('not every dimension requested chunk is full')
  return issues
}

function requestedDimensionCountsByFixture (report) {
  const counts = new Map()
  for (const result of report.results ?? []) {
    const dimensionsByCoordinate = fixtureDimensionsByCoordinate(result, [], 'dimension')
    for (const artifact of result.artifacts ?? []) {
      for (const chunk of artifact.requestedChunks ?? []) {
        const dimension = dimensionsByCoordinate.get(`${chunk.chunkX},${chunk.chunkZ}`) ?? 'overworld'
        counts.set(dimension, (counts.get(dimension) ?? 0) + 1)
      }
    }
  }
  return counts
}

function requestedChunks (report) {
  return (report.results ?? [])
    .flatMap(result => result.artifacts ?? [])
    .flatMap(artifact => artifact.requestedChunks ?? [])
}

function comparableRequestedChunks (report, label = 'fixture') {
  const issues = []
  const chunks = (report?.results ?? []).flatMap(result => {
    const fixtureByCoordinate = fixtureDimensionsByCoordinate(result, issues, label)
    return (result.artifacts ?? []).flatMap(artifact => (artifact.requestedChunks ?? []).flatMap(chunk => {
      const chunkX = chunk.chunkX
      const chunkZ = chunk.chunkZ
      const chunkLabel = `${chunkX ?? '<missing>'},${chunkZ ?? '<missing>'}`
      if (!Number.isInteger(chunkX) || !Number.isInteger(chunkZ)) {
        issues.push(`${label} requested chunk ${chunkLabel} missing integer coordinates`)
        return []
      }
      const dimension = fixtureByCoordinate.get(`${chunk.chunkX},${chunk.chunkZ}`) ?? 'overworld'
      return [{
        ...chunk,
        dimension,
        key: `${dimension}:${chunk.chunkX},${chunk.chunkZ}`
      }]
    }))
  })
  return { chunks, issues }
}

function fixtureDimensionsByCoordinate (result, issues, label) {
  const fixtureByCoordinate = new Map()
  for (const chunk of result.fixture?.chunks ?? []) {
    const key = `${chunk.x},${chunk.z}`
    const dimension = chunk.dimension ?? 'overworld'
    const existing = fixtureByCoordinate.get(key)
    if (existing !== undefined && existing !== dimension) {
      issues.push(`${label} fixture has ambiguous dimensions for chunk ${key}`)
      continue
    }
    fixtureByCoordinate.set(key, dimension)
  }
  return fixtureByCoordinate
}

function combineComparableResults (results) {
  return {
    chunks: results.flatMap(result => result.chunks),
    issues: results.flatMap(result => result.issues)
  }
}

function comparableVibecraftChunks (report) {
  const issues = []
  const chunks = (report?.chunks ?? report?.requestedChunks ?? []).flatMap(chunk => {
    const chunkX = chunk.chunkX ?? chunk.x
    const chunkZ = chunk.chunkZ ?? chunk.z
    const label = `${chunkX ?? '<missing>'},${chunkZ ?? '<missing>'}`
    if (!Number.isInteger(chunkX) || !Number.isInteger(chunkZ)) {
      issues.push(`VibeCraft chunk ${label} missing integer coordinates`)
      return []
    }
    if (typeof chunk.dimension !== 'string' || chunk.dimension.length === 0) {
      issues.push(`VibeCraft chunk ${label} missing dimension`)
      return []
    }
    const dimension = chunk.dimension
    return [{
      ...chunk,
      chunkX,
      chunkZ,
      sectionCount: chunk.sectionCount ?? chunk.section_count,
      nonEmptySectionCount: chunk.nonEmptySectionCount ?? chunk.non_empty_section_count,
      blockPalette: chunk.blockPalette ?? chunk.block_palette,
      biomePalette: chunk.biomePalette ?? chunk.biome_palette,
      payloadSha256: chunk.payloadSha256 ?? chunk.payload_sha256,
      sections: (chunk.sections ?? []).map(section => ({
        ...section,
        blockPalette: section.blockPalette ?? section.block_palette,
        blockStatesData: section.blockStatesData ?? section.block_states_data,
        biomePalette: section.biomePalette ?? section.biome_palette,
        biomeData: section.biomeData ?? section.biome_data
      })),
      dimension,
      key: `${dimension}:${chunkX},${chunkZ}`
    }]
  })
  return { chunks, issues }
}

function stableChunkProjection (chunk) {
  return {
    status: chunk.status,
    sectionCount: chunk.sectionCount,
    nonEmptySectionCount: chunk.nonEmptySectionCount,
    heightmaps: sortedObject(chunk.heightmaps ?? {}),
    structures: {
      startKeys: [...(chunk.structures?.startKeys ?? [])].sort(),
      referenceKeys: [...(chunk.structures?.referenceKeys ?? [])].sort()
    },
    blockPalette: [...(chunk.blockPalette ?? [])].sort(),
    biomePalette: [...(chunk.biomePalette ?? [])].sort(),
    sections: (chunk.sections ?? []).map(section => ({
      y: section.y,
      blockPalette: [...(section.blockPalette ?? [])].sort(),
      blockStatesData: section.blockStatesData ?? null,
      biomePalette: [...(section.biomePalette ?? [])].sort(),
      biomeData: section.biomeData ?? null
    })).sort((left, right) => left.y - right.y),
    payloadSha256: chunk.payloadSha256
  }
}

function sortedObject (object) {
  return Object.fromEntries(Object.entries(object).sort(([left], [right]) => left.localeCompare(right)))
}

async function readOptionalJson (filePath, required) {
  try {
    return JSON.parse(await readFile(filePath, 'utf8'))
  } catch (error) {
    if (!required && error?.code === 'ENOENT') return undefined
    throw error
  }
}

function runShellCommand (command, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, {
      cwd,
      shell: true,
      stdio: ['ignore', 'inherit', 'inherit']
    })
    child.on('error', reject)
    child.on('exit', code => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`command failed with exit ${code}: ${command}`))
      }
    })
  })
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const report = await loadWorldgenAcceptanceReport({
    requireReports: process.env.VIBECRAFT_REQUIRE_WORLDGEN_REPORTS === '1'
  })
  if (process.env.VIBECRAFT_COMPARE_OVERWORLD_FIXTURE_REPORT) {
    report.overworldStability = compareWorldgenFixtureReports(
      await readOptionalJson(report.overworldPath, true),
      await readOptionalJson(process.env.VIBECRAFT_COMPARE_OVERWORLD_FIXTURE_REPORT, true)
    )
  }
  if (process.env.VIBECRAFT_COMPARE_DIMENSION_FIXTURE_REPORT) {
    report.dimensionStability = compareWorldgenFixtureReports(
      await readOptionalJson(report.dimensionPath, true),
      await readOptionalJson(process.env.VIBECRAFT_COMPARE_DIMENSION_FIXTURE_REPORT, true)
    )
  }
  if (process.env.VIBECRAFT_RUST_WORLDGEN_REPORT) {
    report.vibecraftComparison = compareVibecraftWorldgenReport(
      [
        await readOptionalJson(report.overworldPath, true),
        await readOptionalJson(report.dimensionPath, true)
      ],
      await readOptionalJson(process.env.VIBECRAFT_RUST_WORLDGEN_REPORT, true)
    )
  }
  if (process.env.VIBECRAFT_GENERATE_RUST_WORLDGEN_REPORT === '1') {
    report.vibecraftReportPath = await generateVibecraftWorldgenReport()
    report.vibecraftComparison = compareVibecraftWorldgenReport(
      [
        await readOptionalJson(report.overworldPath, true),
        await readOptionalJson(report.dimensionPath, true)
      ],
      await readOptionalJson(report.vibecraftReportPath, true)
    )
  }
  console.log(JSON.stringify(report, null, 2))
  const target = process.env.VIBECRAFT_WORLDGEN_ACCEPT_THROUGH
  if (target && report.completeThrough !== target) {
    console.error(`worldgen acceptance stopped at ${report.completeThrough}, expected ${target}`)
    process.exit(1)
  }
  if (report.overworldStability && !report.overworldStability.ok) process.exit(1)
  if (report.dimensionStability && !report.dimensionStability.ok) process.exit(1)
  if (report.vibecraftComparison && !report.vibecraftComparison.ok) process.exit(1)
}
