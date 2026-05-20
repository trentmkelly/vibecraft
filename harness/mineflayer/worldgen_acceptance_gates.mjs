import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

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
    format: 'rustcraft-worldgen-acceptance-gates-v1',
    phases,
    completeThrough: completeThrough(phases)
  }
}

export async function loadWorldgenAcceptanceReport ({
  overworldPath = process.env.RUSTCRAFT_OVERWORLD_FIXTURE_REPORT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures.json'),
  dimensionPath = process.env.RUSTCRAFT_DIMENSION_FIXTURE_REPORT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-dimension-fixtures.json'),
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
  const leftChunks = comparableRequestedChunks(left)
  const rightChunks = comparableRequestedChunks(right)
  return compareComparableChunkLists(leftChunks, rightChunks, { leftLabel: 'left', rightLabel: 'right' })
}

export function compareRustcraftWorldgenReport (vanillaReports, rustcraftReport) {
  const reports = Array.isArray(vanillaReports) ? vanillaReports : [vanillaReports]
  const vanillaChunks = reports.flatMap(report => comparableRequestedChunks(report))
  const rustcraftChunks = comparableRustcraftChunks(rustcraftReport)
  return compareComparableChunkLists(vanillaChunks, rustcraftChunks, {
    leftLabel: 'vanilla',
    rightLabel: 'rustcraft'
  })
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
  const issues = []
  if (chunks.length === 0) issues.push('overworld report has no requested chunks')
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
  const issues = []
  if (!dimensions.has('the_nether')) issues.push('dimension report missing the_nether fixture')
  if (!dimensions.has('the_end')) issues.push('dimension report missing the_end fixture')
  if (chunks.length === 0) issues.push('dimension report has no requested chunks')
  if (!chunks.every(chunk => chunk.status === 'minecraft:full')) issues.push('not every dimension requested chunk is full')
  return issues
}

function requestedChunks (report) {
  return (report.results ?? [])
    .flatMap(result => result.artifacts ?? [])
    .flatMap(artifact => artifact.requestedChunks ?? [])
}

function comparableRequestedChunks (report) {
  return (report?.results ?? []).flatMap(result => {
    const fixtureByCoordinate = new Map(
      (result.fixture?.chunks ?? []).map(chunk => [`${chunk.x},${chunk.z}`, chunk.dimension ?? 'overworld'])
    )
    return (result.artifacts ?? []).flatMap(artifact => (artifact.requestedChunks ?? []).map(chunk => ({
      ...chunk,
      dimension: fixtureByCoordinate.get(`${chunk.chunkX},${chunk.chunkZ}`) ?? 'overworld',
      key: `${fixtureByCoordinate.get(`${chunk.chunkX},${chunk.chunkZ}`) ?? 'overworld'}:${chunk.chunkX},${chunk.chunkZ}`
    })))
  })
}

function comparableRustcraftChunks (report) {
  return (report?.chunks ?? report?.requestedChunks ?? []).map(chunk => {
    const chunkX = chunk.chunkX ?? chunk.x
    const chunkZ = chunk.chunkZ ?? chunk.z
    const dimension = chunk.dimension ?? 'overworld'
    return {
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
    }
  })
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

if (import.meta.url === `file://${process.argv[1]}`) {
  const report = await loadWorldgenAcceptanceReport({
    requireReports: process.env.RUSTCRAFT_REQUIRE_WORLDGEN_REPORTS === '1'
  })
  if (process.env.RUSTCRAFT_COMPARE_OVERWORLD_FIXTURE_REPORT) {
    report.overworldStability = compareWorldgenFixtureReports(
      await readOptionalJson(report.overworldPath, true),
      await readOptionalJson(process.env.RUSTCRAFT_COMPARE_OVERWORLD_FIXTURE_REPORT, true)
    )
  }
  if (process.env.RUSTCRAFT_COMPARE_DIMENSION_FIXTURE_REPORT) {
    report.dimensionStability = compareWorldgenFixtureReports(
      await readOptionalJson(report.dimensionPath, true),
      await readOptionalJson(process.env.RUSTCRAFT_COMPARE_DIMENSION_FIXTURE_REPORT, true)
    )
  }
  if (process.env.RUSTCRAFT_RUST_WORLDGEN_REPORT) {
    report.rustcraftComparison = compareRustcraftWorldgenReport(
      [
        await readOptionalJson(report.overworldPath, true),
        await readOptionalJson(report.dimensionPath, true)
      ],
      await readOptionalJson(process.env.RUSTCRAFT_RUST_WORLDGEN_REPORT, true)
    )
  }
  console.log(JSON.stringify(report, null, 2))
  const target = process.env.RUSTCRAFT_WORLDGEN_ACCEPT_THROUGH
  if (target && report.completeThrough !== target) {
    console.error(`worldgen acceptance stopped at ${report.completeThrough}, expected ${target}`)
    process.exit(1)
  }
  if (report.overworldStability && !report.overworldStability.ok) process.exit(1)
  if (report.dimensionStability && !report.dimensionStability.ok) process.exit(1)
  if (report.rustcraftComparison && !report.rustcraftComparison.ok) process.exit(1)
}
