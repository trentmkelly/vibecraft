export const PARITY_COVERAGE_CATALOG = {
  protocol: [
    '182-game-packets',
    'packet-family-metadata',
    'clientbound-golden-serialization',
    'serverbound-fuzz-and-replay'
  ],
  blockEntities: [
    'base-block-entity',
    'furnace-family',
    'container-family',
    'randomizable-container-mineflayer',
    'sign-text-family',
    'utility-family',
    'world-system-family',
    'decorative-lore-family',
    'mob-environment-family',
    'every-block-entity-test-matrix'
  ],
  menus: [
    'abstract-container-menu',
    'crafting-player-menus',
    'furnace-menus',
    'workstation-menus',
    'storage-transfer-menus',
    'merchant-menu',
    'merchant-menu-mineflayer',
    'every-menu-click-matrix'
  ],
  recipes: [
    'recipe-manager',
    'core-recipe-interfaces',
    'grid-crafting',
    'special-crafting',
    'cooking-recipes',
    'smithing-station-recipes',
    'recipe-validation-matrix'
  ],
  entityFamilies: [
    'boss-families',
    'monster-families',
    'npc-families',
    'player-entity-specifics',
    'raid-package',
    'entity-family-parity-scenarios'
  ],
  worldgen: [
    'coarse-row-reaudit',
    'real-generated-chunk-packets',
    'bootstrap-graph',
    'world-preset-dimension-stem-loading',
    'random-stack',
    'biome-source-codecs',
    'climate-sampler',
    'biome-generation-settings',
    'chunk-status-pipeline',
    'generator-methods',
    'noise-settings-router-loading',
    'density-function-runtime',
    'noise-samplers',
    'surface-height-column-queries',
    'aquifer-fluid-picker',
    'ore-veinifier',
    'surface-rules',
    'carvers',
    'feature-primitives',
    'biome-decoration-ordering',
    'structure-set-placement',
    'structure-starts',
    'non-jigsaw-structures',
    'jigsaw-template-structures',
    'data-driven-jigsaw-content',
    'terrain-blending-retrogen',
    'spawn-position-parity',
    'chunk-generation-mobs',
    'generated-output-completeness',
    'lighting-handoff',
    'region-persistence-compatibility',
    'java-guided-trace-harness',
    'overworld-fixtures',
    'nether-end-fixtures',
    'progressive-acceptance-gates'
  ],
  storageOps: [
    'level-storage-source',
    'saved-data-storage',
    'datafix-strategy',
    'operational-files',
    'json-rpc-management',
    'game-test-hooks',
    'generated-report-tooling'
  ],
  milestonesAndRelease: [
    'milestone-4-player',
    'milestone-5-block-inventory',
    'milestone-6-commands',
    'milestone-7-region-lighting',
    'milestone-8-datapacks',
    'milestone-9-worldgen',
    'milestone-10-entities',
    'milestone-11-command-parity',
    'milestone-12-online-secure-chat',
    'milestone-13-management-and-26-1-2',
    'milestone-14-black-box-parity',
    'milestone-15-production-hardening',
    'vanilla-client-join-play-reconnect',
    'deterministic-worldgen',
    'official-world-loading',
    'server-properties-operational-files',
    'protocol-states-packets',
    'command-permission-side-effects',
    'content-parity-tests',
    'official-vs-rebuilt-harness',
    'known-differences-documented'
  ]
}

export function createParityCoveragePlan(kind) {
  const steps = PARITY_COVERAGE_CATALOG[kind]
  if (!steps) throw new Error(`Unknown parity coverage catalog ${kind}`)
  return { name: `parity-coverage-${kebab(kind)}`, kind, steps }
}

export function summarizeParityCoverage(evidence, plan) {
  const observed = new Set((evidence.timeline ?? [])
    .filter(event => event.name === 'parity_coverage')
    .map(event => event.summary?.[0]))
  const steps = Object.fromEntries(plan.steps.map(step => [step, observed.has(step)]))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function recordParityCoverage(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'parity_coverage', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

export async function runParityCoverageCatalog(kind, options = {}) {
  const plan = createParityCoveragePlan(kind)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? coverageProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing parity coverage evidence for ${step}`)
    recordParityCoverage(evidence, step, result.details?.[step] ?? {})
  }
  return { plan, evidence, summary: summarizeParityCoverage(evidence, plan) }
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function coverageProbe() {
  throw new Error('coverageProbe requires a scenario-specific verifier')
}
