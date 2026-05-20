import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { runVanillaWorldgenOracle } from './vanilla_worldgen_oracle.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

export const OVERWORLD_PARITY_MATRIX_SEEDS = [0n, 1n, -1n, 2147483647n]
export const OVERWORLD_PARITY_MATRIX_CHUNKS = [
  { x: 0, z: 0 },
  { x: 1, z: 0 },
  { x: 0, z: 1 },
  { x: 16, z: 16 }
]

export const DIMENSION_PARITY_MATRIX_SEEDS = [0n, 1n, -1n, 2147483647n]
export const DIMENSION_PARITY_MATRIX_CHUNKS = {
  the_nether: [
    { x: 0, z: 0, dimension: 'the_nether' },
    { x: 1, z: 0, dimension: 'the_nether' },
    { x: 8, z: -8, dimension: 'the_nether' },
    { x: 32, z: 32, dimension: 'the_nether' }
  ],
  the_end: [
    { x: 0, z: 0, dimension: 'the_end' },
    { x: 1, z: 0, dimension: 'the_end' },
    { x: 8, z: 8, dimension: 'the_end' },
    { x: 64, z: 0, dimension: 'the_end' }
  ]
}

export const OVERWORLD_FIXTURE_CASES = [
  ...OVERWORLD_PARITY_MATRIX_SEEDS.map(seed => ({
    id: `overworld_parity_matrix_seed_${fixtureSeedId(seed)}`,
    category: 'explicit_overworld_seed_coordinate_matrix',
    seed,
    chunks: OVERWORLD_PARITY_MATRIX_CHUNKS,
    notes: 'Explicit checklist matrix covering chunks (0,0), (1,0), (0,1), and (16,16) for deterministic heightmap, biome, section palette, and representative block-position parity.'
  })),
  {
    id: 'overworld_forest_spawn_origin',
    category: 'plains_forest_spawn',
    seed: 8675309n,
    chunks: [{ x: 0, z: 0 }, { x: 1, z: 0 }, { x: -1, z: -1 }],
    notes: 'Origin-adjacent forest/plains spawn chunks with caves, ores, mineshaft references, fluids, trees, and surface vegetation.'
  },
  {
    id: 'overworld_negative_region_boundary',
    category: 'region_boundary_negative_coords',
    seed: 8675309n,
    chunks: [{ x: -1, z: 0 }, { x: 0, z: -1 }, { x: -32, z: -32 }],
    notes: 'Negative chunk and region-boundary coverage for floor division, region path selection, and persisted xPos/zPos handling.'
  },
  {
    id: 'overworld_far_noise_sample',
    category: 'far_noise_sample',
    seed: 123456789n,
    chunks: [{ x: 64, z: 64 }, { x: -64, z: 32 }, { x: 96, z: -96 }],
    notes: 'Far coordinate noise sample to expose large-coordinate random, climate, surface, aquifer, and carver drift.'
  }
]

export const DIMENSION_FIXTURE_CASES = [
  ...Object.entries(DIMENSION_PARITY_MATRIX_CHUNKS).flatMap(([dimension, chunks]) =>
    DIMENSION_PARITY_MATRIX_SEEDS.map(seed => ({
      id: `${dimension}_parity_matrix_seed_${fixtureSeedId(seed)}`,
      category: `${dimension}_multi_seed_coordinate_matrix`,
      seed,
      chunks,
      notes: dimension === 'the_nether'
        ? 'Multi-seed Nether matrix covering origin, nearby, far, and mixed-sign chunks for biome source, density, lava/air distribution, features, and structures parity.'
        : 'Multi-seed End matrix covering origin, spawn-platform-adjacent, near island, and far island chunks for biome source, terrain density, features, structures, and spawn-platform parity.'
    }))
  ),
  {
    id: 'nether_origin_smoke',
    category: 'nether_origin',
    seed: 8675309n,
    chunks: [{ x: 0, z: 0, dimension: 'the_nether' }, { x: 1, z: 0, dimension: 'the_nether' }],
    notes: 'Nether origin chunks for biome source selection, density, lava/air distribution, features, and structure reference smoke coverage.'
  },
  {
    id: 'end_origin_smoke',
    category: 'end_origin',
    seed: 8675309n,
    chunks: [{ x: 0, z: 0, dimension: 'the_end' }, { x: 1, z: 0, dimension: 'the_end' }],
    notes: 'End origin chunks for biome source selection, island density, features, structures, and spawn-platform-adjacent smoke coverage.'
  }
]

export function fixtureManifest ({ includePendingDimensions = true } = {}) {
  const cases = includePendingDimensions
    ? [...OVERWORLD_FIXTURE_CASES, ...DIMENSION_FIXTURE_CASES]
    : [...OVERWORLD_FIXTURE_CASES]
  return {
    format: 'rustcraft-vanilla-worldgen-fixtures-v1',
    generatedBy: 'harness/mineflayer/vanilla_worldgen_fixtures.mjs',
    cases: cases.map(serializeFixtureCase)
  }
}

export async function runOverworldFixtureSuite ({
  root = path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures'),
  output = path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures.json'),
  cases = OVERWORLD_FIXTURE_CASES,
  timeoutMs = 120_000
} = {}) {
  const results = []
  for (const fixture of cases) {
    const result = await runVanillaWorldgenOracle({
      root: path.join(root, fixture.id),
      seed: fixture.seed,
      chunks: fixture.chunks,
      timeoutMs
    })
    results.push({
      fixture: serializeFixtureCase(fixture),
      ok: result.ok,
      plan: result.plan,
      artifacts: result.artifacts,
      logTail: result.logTail
    })
  }
  const report = {
    ...fixtureManifest({ includePendingDimensions: false }),
    results
  }
  await mkdir(path.dirname(output), { recursive: true })
  await writeFile(output, `${JSON.stringify(report, null, 2)}\n`)
  return report
}

export async function runDimensionFixtureSuite ({
  root = path.join(repoRoot, 'target', 'vanilla-worldgen-dimension-fixtures'),
  output = path.join(repoRoot, 'target', 'vanilla-worldgen-dimension-fixtures.json'),
  cases = DIMENSION_FIXTURE_CASES,
  timeoutMs = 120_000,
  runOracle = options => runVanillaWorldgenOracle(options)
} = {}) {
  const results = []
  for (const fixture of cases) {
    const result = await runOracle({
      fixture,
      root: path.join(root, fixture.id),
      seed: fixture.seed,
      chunks: fixture.chunks,
      timeoutMs
    })
    results.push({
      fixture: serializeFixtureCase(fixture),
      ok: result.ok,
      plan: result.plan,
      artifacts: result.artifacts,
      logTail: result.logTail
    })
  }
  const report = {
    format: 'rustcraft-vanilla-worldgen-dimension-fixtures-v1',
    generatedBy: 'harness/mineflayer/vanilla_worldgen_fixtures.mjs',
    cases: cases.map(serializeFixtureCase),
    results
  }
  await mkdir(path.dirname(output), { recursive: true })
  await writeFile(output, `${JSON.stringify(report, null, 2)}\n`)
  return report
}

function serializeFixtureCase (fixture) {
  return {
    ...fixture,
    seed: fixture.seed.toString(),
    chunks: fixture.chunks.map(chunk => ({
      x: Number(chunk.x),
      z: Number(chunk.z),
      dimension: chunk.dimension ?? 'overworld'
    }))
  }
}

function fixtureSeedId (seed) {
  return seed < 0n ? `neg_${(-seed).toString()}` : seed.toString()
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const root = process.env.RUSTCRAFT_VANILLA_FIXTURE_ROOT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures')
  const output = process.env.RUSTCRAFT_VANILLA_FIXTURE_OUTPUT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures.json')
  const onlyManifest = process.env.RUSTCRAFT_VANILLA_FIXTURE_MANIFEST_ONLY === '1'
  const dimensionsOnly = process.env.RUSTCRAFT_VANILLA_FIXTURE_DIMENSIONS === '1'
  const report = onlyManifest
    ? fixtureManifest()
    : dimensionsOnly
      ? await runDimensionFixtureSuite({ root, output })
    : await runOverworldFixtureSuite({ root, output })
  if (onlyManifest) {
    console.log(JSON.stringify(report, null, 2))
  } else {
    console.log(JSON.stringify({ output, ok: report.results.every(result => result.ok), cases: report.results.length }, null, 2))
  }
}
