import { offlineUuid } from './runner.mjs'

/// Mineflayer scenarios that exercise recipe-book unlocking + crafting against a
/// live VibeCraft server in offline mode: receive the recipe book on join, craft
/// in the 2×2 player grid and the 3×3 crafting table, open a workstation, and
/// confirm the result slot + recipe-sync packets match vanilla.
///
/// Each scenario fixes the recipe it crafts and the grid/workstation it uses; the
/// plan steps are granular so a future rig can independently answer "did the
/// server send UpdateRecipes / RecipeBookAdd?", "did the result slot populate?",
/// and "was the recipe marked unlocked after the first craft?".
export const CRAFTING_RECIPE_SCENARIOS = {
  // 2×2 player-inventory crafting (InventoryMenu).
  playerGrid2x2: { grid: '2x2', workstation: 'playerInventory', resultRecipe: 'minecraft:oak_planks' },
  // 3×3 crafting table (CraftingMenu).
  craftingTable3x3: { grid: '3x3', workstation: 'craftingTable', resultRecipe: 'minecraft:crafting_table' },
  // First-craft recipe-book unlock + highlight.
  recipeBookUnlock: { grid: '3x3', workstation: 'craftingTable', resultRecipe: 'minecraft:chest' },
  // Opening a workstation (stonecutter) and selecting a recipe.
  stonecutterSelect: { grid: 'workstation', workstation: 'stonecutter', resultRecipe: 'minecraft:stone_slab' }
}

export const CRAFTING_RECIPE_STEPS = [
  'join-and-receive-recipe-book',
  'observe-update-recipes-sync',
  'open-crafting-grid',
  'place-ingredients',
  'observe-result-slot',
  'take-result',
  'observe-recipe-unlocked',
  'observe-recipe-book-add'
]

export function createCraftingRecipePlan(scenario, options = {}) {
  const config = CRAFTING_RECIPE_SCENARIOS[scenario]
  if (!config) throw new Error(`Unknown crafting recipe scenario ${scenario}`)
  const username = options.username ?? defaultUsername(scenario)
  return {
    name: `mineflayer-crafting-recipe-${kebab(scenario)}`,
    scenario,
    mode: 'offline',
    auth: 'offline',
    username,
    uuid: offlineUuid(username),
    grid: config.grid,
    workstation: config.workstation,
    resultRecipe: config.resultRecipe,
    steps: CRAFTING_RECIPE_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      gamemode: options.gamemode ?? 'survival'
    }
  }
}

export async function runCraftingRecipeScenario(scenario, options = {}) {
  const plan = createCraftingRecipePlan(scenario, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? craftingRecipeProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing crafting recipe evidence for ${scenario}:${step}`)
    recordCraftingRecipeEvent(evidence, step, result.details?.[step] ?? {})
  }
  // Sanity check: the probe must report the crafted result item, proving it
  // actually observed the result slot rather than echoing the step names.
  if (result.resultItem !== plan.resultRecipe) {
    throw new Error(
      `Result mismatch for ${scenario}: probe reported ${result.resultItem}, expected ${plan.resultRecipe}`
    )
  }
  return {
    plan,
    evidence,
    summary: summarizeCraftingRecipeEvidence(evidence, plan)
  }
}

export function summarizeCraftingRecipeEvidence(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'crafting_recipe')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordCraftingRecipeEvent(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'crafting_recipe', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function defaultUsername(scenario) {
  return {
    playerGrid2x2: 'CraftGrid2x2Bot',
    craftingTable3x3: 'CraftTable3x3Bot',
    recipeBookUnlock: 'RecipeUnlockBot',
    stonecutterSelect: 'StonecutterBot'
  }[scenario] ?? 'CraftRecipeBot'
}

function kebab(value) {
  return value.replace(/[A-Z0-9]/g, letter => `-${letter.toLowerCase()}`)
}

async function craftingRecipeProbe() {
  throw new Error(
    'craftingRecipeProbe requires a live VibeCraft server with an in-world crafting table / workstation to interact with; pass an `options.probe` override to drive the scenario manually'
  )
}

if (import.meta.url === `file://${process.argv[1]}`) {
  for (const scenario of Object.keys(CRAFTING_RECIPE_SCENARIOS)) {
    console.log(JSON.stringify(createCraftingRecipePlan(scenario), null, 2))
  }
}
