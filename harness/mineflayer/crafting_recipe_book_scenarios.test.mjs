import test from 'node:test'
import assert from 'node:assert/strict'
import {
  CRAFTING_RECIPE_SCENARIOS,
  CRAFTING_RECIPE_STEPS,
  createCraftingRecipePlan,
  recordCraftingRecipeEvent,
  runCraftingRecipeScenario,
  summarizeCraftingRecipeEvidence
} from './crafting_recipe_book_scenarios.mjs'
import { offlineUuid } from './runner.mjs'

for (const scenario of Object.keys(CRAFTING_RECIPE_SCENARIOS)) {
  test(`createCraftingRecipePlan covers ${scenario}`, () => {
    const plan = createCraftingRecipePlan(scenario)
    assert.equal(plan.scenario, scenario)
    assert.equal(plan.mode, 'offline')
    assert.equal(plan.auth, 'offline')
    assert.equal(plan.uuid, offlineUuid(plan.username))
    assert.equal(plan.resultRecipe, CRAFTING_RECIPE_SCENARIOS[scenario].resultRecipe)
    assert.equal(plan.grid, CRAFTING_RECIPE_SCENARIOS[scenario].grid)
    assert.deepEqual(plan.steps, CRAFTING_RECIPE_STEPS)
    assert.equal(plan.serverProperties['online-mode'], 'false')
  })

  test(`runCraftingRecipeScenario validates ${scenario} when probe reports it`, async () => {
    const result = await runCraftingRecipeScenario(scenario, {
      probe: async plan => ({
        resultItem: plan.resultRecipe,
        steps: Object.fromEntries(plan.steps.map(step => [step, true])),
        details: Object.fromEntries(plan.steps.map(step => [step, { observed: true }]))
      })
    })
    assert.equal(result.summary.ok, true)
    for (const step of result.plan.steps) {
      assert.equal(result.summary.steps[step], true)
    }
  })
}

test('createCraftingRecipePlan throws on unknown scenario', () => {
  assert.throws(() => createCraftingRecipePlan('enchanting'), /Unknown crafting recipe scenario/)
})

test('summarizeCraftingRecipeEvidence fails on missing evidence', () => {
  const plan = createCraftingRecipePlan('craftingTable3x3')
  const evidence = { timeline: [] }
  recordCraftingRecipeEvent(evidence, plan.steps[0])
  const summary = summarizeCraftingRecipeEvidence(evidence, plan)
  assert.equal(summary.ok, false)
  assert.equal(summary.steps[plan.steps[0]], true)
})

test('runCraftingRecipeScenario fails closed when the crafted result differs', async () => {
  await assert.rejects(
    () => runCraftingRecipeScenario('playerGrid2x2', {
      probe: async plan => ({
        resultItem: 'minecraft:stone', // wrong result → must fail closed
        steps: Object.fromEntries(plan.steps.map(step => [step, true]))
      })
    }),
    /Result mismatch/
  )
})

test('default probe rejects until a server fixture wires the crafting grid', async () => {
  await assert.rejects(
    () => runCraftingRecipeScenario('craftingTable3x3'),
    /requires a live RustCraft server/
  )
})
