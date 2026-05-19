import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildRecipeBookEvidence,
  createRecipeBookScenarioPlan,
  summarizeRecipeBookEvidence
} from './recipe_scenarios.mjs'

test('createRecipeBookScenarioPlan covers crafting, recipe sync, workstations, and vanilla diff', () => {
  const plan = createRecipeBookScenarioPlan()

  assert.equal(plan.name, 'mineflayer-recipe-book-crafting')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.deepEqual(plan.required, [
    'recipes-unlocked',
    'crafts-2x2-grid',
    'crafts-3x3-grid',
    'opens-workstation',
    'recipe-sync-verified',
    'result-slots-match-vanilla',
    'official-server-diff'
  ])
})

test('summarizeRecipeBookEvidence fails closed on missing recipe scenario evidence', () => {
  const plan = createRecipeBookScenarioPlan()
  const complete = Object.fromEntries(plan.required.map(key => [key, true]))

  assert.equal(summarizeRecipeBookEvidence(complete, plan).ok, true)

  const incomplete = summarizeRecipeBookEvidence({ 'recipes-unlocked': true }, plan)
  assert.equal(incomplete.ok, false)
  assert.ok(incomplete.missing.includes('crafts-2x2-grid'))
  assert.ok(incomplete.missing.includes('official-server-diff'))
})

test('buildRecipeBookEvidence maps Mineflayer observations to checklist evidence', () => {
  const evidence = buildRecipeBookEvidence({
    unlockedRecipes: ['minecraft:oak_planks'],
    craftedGrids: ['2x2', '3x3'],
    openedWorkstations: ['minecraft:crafting_table'],
    packets: [{ name: 'recipe_book_add' }],
    resultSlotDiff: { ok: true },
    officialDiff: { ok: true }
  })

  assert.equal(evidence['recipes-unlocked'], true)
  assert.equal(evidence['crafts-2x2-grid'], true)
  assert.equal(evidence['crafts-3x3-grid'], true)
  assert.equal(evidence['opens-workstation'], true)
  assert.equal(evidence['recipe-sync-verified'], true)
  assert.equal(evidence['result-slots-match-vanilla'], true)
  assert.equal(evidence['official-server-diff'], true)

  assert.equal(buildRecipeBookEvidence({ packets: [] })['recipe-sync-verified'], false)
})
