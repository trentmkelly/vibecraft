export function createRecipeBookScenarioPlan() {
  return {
    name: 'mineflayer-recipe-book-crafting',
    mode: 'offline',
    auth: 'offline',
    required: [
      'recipes-unlocked',
      'crafts-2x2-grid',
      'crafts-3x3-grid',
      'opens-workstation',
      'recipe-sync-verified',
      'result-slots-match-vanilla',
      'official-server-diff'
    ]
  }
}

export function summarizeRecipeBookEvidence(evidence, plan = createRecipeBookScenarioPlan()) {
  const missing = plan.required.filter(key => !evidence[key])
  return {
    ok: missing.length === 0,
    missing
  }
}

export function buildRecipeBookEvidence(observations) {
  return {
    'recipes-unlocked': observations.unlockedRecipes?.length > 0,
    'crafts-2x2-grid': observations.craftedGrids?.includes('2x2') === true,
    'crafts-3x3-grid': observations.craftedGrids?.includes('3x3') === true,
    'opens-workstation': observations.openedWorkstations?.length > 0,
    'recipe-sync-verified': observations.packets?.some(packet => packet.name === 'update_recipes' || packet.name === 'recipe_book_add') === true,
    'result-slots-match-vanilla': observations.resultSlotDiff?.ok === true,
    'official-server-diff': observations.officialDiff?.ok === true
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createRecipeBookScenarioPlan(), null, 2))
}
