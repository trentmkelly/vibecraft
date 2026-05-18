import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { runVanillaWorldgenOracle } from './vanilla_worldgen_oracle.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

export const OVERWORLD_FIXTURE_CASES = [
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
  {
    id: 'nether_origin_smoke',
    category: 'nether_origin',
    seed: 8675309n,
    chunks: [{ x: 0, z: 0, dimension: 'the_nether' }],
    notes: 'Fixture definition only for now; the current oracle command path force-loads overworld chunks and still needs dimension-aware execution.'
  },
  {
    id: 'end_origin_smoke',
    category: 'end_origin',
    seed: 8675309n,
    chunks: [{ x: 0, z: 0, dimension: 'the_end' }],
    notes: 'Fixture definition only for now; the current oracle command path force-loads overworld chunks and still needs dimension-aware execution.'
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

if (import.meta.url === `file://${process.argv[1]}`) {
  const root = process.env.RUSTCRAFT_VANILLA_FIXTURE_ROOT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures')
  const output = process.env.RUSTCRAFT_VANILLA_FIXTURE_OUTPUT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-fixtures.json')
  const onlyManifest = process.env.RUSTCRAFT_VANILLA_FIXTURE_MANIFEST_ONLY === '1'
  const report = onlyManifest
    ? fixtureManifest()
    : await runOverworldFixtureSuite({ root, output })
  if (onlyManifest) {
    console.log(JSON.stringify(report, null, 2))
  } else {
    console.log(JSON.stringify({ output, ok: report.results.every(result => result.ok), cases: report.results.length }, null, 2))
  }
}
