import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildGameruleEvidence,
  buildScoreboardObjectiveEvidence,
  buildScoreboardTeamEvidence,
  buildStatsAdvancementsEvidence,
  createGameStateScenarioPlan,
  summarizeGameStateEvidence
} from './game_state_scenarios.mjs'

test('createGameStateScenarioPlan covers gamerule, stats, advancement, scoreboard, and team workflows', () => {
  const plan = createGameStateScenarioPlan()

  assert.equal(plan.name, 'mineflayer-game-state-scenarios')
  assert.equal(plan.mode, 'offline')
  assert.equal(plan.auth, 'offline')
  assert.deepEqual(plan.scenarios.map(scenario => scenario.name), [
    'gamerules',
    'statsAdvancements',
    'scoreboardObjectives',
    'scoreboardTeams'
  ])
})

test('summarizeGameStateEvidence passes complete evidence and reports missing scenario surfaces', () => {
  const plan = createGameStateScenarioPlan()
  const complete = Object.fromEntries(plan.scenarios.map(scenario => [
    scenario.name,
    Object.fromEntries(scenario.required.map(key => [key, true]))
  ]))

  assert.equal(summarizeGameStateEvidence(complete, plan).ok, true)

  const incomplete = summarizeGameStateEvidence({
    gamerules: { 'toggle-keepInventory-and-die': true }
  }, plan)
  assert.equal(incomplete.ok, false)
  assert.ok(incomplete.scenarios.find(result => result.name === 'gamerules').missing.includes('toggle-doImmediateRespawn-and-die'))
  assert.ok(incomplete.scenarios.find(result => result.name === 'statsAdvancements').missing.includes('saved-stats-json-after-reconnect'))
})

test('buildGameruleEvidence validates client-visible effects for the requested gamerules', () => {
  const evidence = buildGameruleEvidence({
    keepInventory: { inventoryRetained: true },
    doImmediateRespawn: { deathScreenSkipped: true },
    sendCommandFeedback: { feedbackSuppressed: true },
    doDaylightCycle: { timeStopped: true },
    mobGriefing: { worldUnchanged: true },
    vanillaDiff: { ok: true }
  })

  assert.equal(evidence['toggle-keepInventory-and-die'], true)
  assert.equal(evidence['toggle-doImmediateRespawn-and-die'], true)
  assert.equal(evidence['toggle-sendCommandFeedback-and-run-command'], true)
  assert.equal(evidence['toggle-doDaylightCycle-and-observe-time'], true)
  assert.equal(evidence['toggle-mobGriefing-and-trigger-griefing-mob'], true)
  assert.equal(evidence['client-observable-behavior-diffed-against-vanilla'], true)

  assert.equal(buildGameruleEvidence({})['toggle-keepInventory-and-die'], false)
})

test('buildStatsAdvancementsEvidence requires actions, client packets, and saved JSON after reconnect', () => {
  const evidence = buildStatsAdvancementsEvidence({
    actions: ['movement', 'mining', 'crafting', 'death', 'recipe-unlock'],
    packets: [{ name: 'award_stats' }, { name: 'update_advancements' }],
    savedFiles: {
      statsJsonAfterReconnect: true,
      advancementsJsonAfterReconnect: true
    }
  })

  assert.equal(evidence['movement-stat-action'], true)
  assert.equal(evidence['mining-stat-action'], true)
  assert.equal(evidence['crafting-stat-action'], true)
  assert.equal(evidence['death-stat-action'], true)
  assert.equal(evidence['recipe-unlock-action'], true)
  assert.equal(evidence['client-stats-packet-observed'], true)
  assert.equal(evidence['client-advancement-packet-observed'], true)
  assert.equal(evidence['saved-stats-json-after-reconnect'], true)
  assert.equal(evidence['saved-advancements-json-after-reconnect'], true)

  assert.equal(buildStatsAdvancementsEvidence({ actions: ['movement'], packets: [] })['client-stats-packet-observed'], false)
})

test('buildScoreboardObjectiveEvidence covers objective lifecycle, display slots, and persistence', () => {
  const evidence = buildScoreboardObjectiveEvidence({
    commands: [
      'scoreboard objectives add kills dummy',
      'scoreboard players set Steve kills 3',
      'scoreboard objectives setdisplay sidebar kills',
      'scoreboard objectives remove kills'
    ],
    displays: ['sidebar', 'list', 'below_name'],
    persistence: { objectivesAfterReconnect: true }
  })

  assert.equal(evidence['objective-create'], true)
  assert.equal(evidence['score-update'], true)
  assert.equal(evidence['display-sidebar'], true)
  assert.equal(evidence['display-list'], true)
  assert.equal(evidence['display-below-name'], true)
  assert.equal(evidence['display-hide'], true)
  assert.equal(evidence['objective-remove'], true)
  assert.equal(evidence['persistence-after-reconnect'], true)
})

test('buildScoreboardTeamEvidence covers display slots, formatted names, rules, and persistence', () => {
  const evidence = buildScoreboardTeamEvidence({
    displays: ['sidebar', 'list', 'below_name'],
    team: {
      colorVisible: true,
      prefixVisible: true,
      suffixVisible: true,
      nametagVisibilityMatchesVanilla: true,
      collisionMatchesVanilla: true
    },
    persistence: { teamsAfterReconnect: true }
  })

  assert.equal(evidence['sidebar-display-visible'], true)
  assert.equal(evidence['list-display-visible'], true)
  assert.equal(evidence['below-name-display-visible'], true)
  assert.equal(evidence['team-color-visible'], true)
  assert.equal(evidence['team-prefix-visible'], true)
  assert.equal(evidence['team-suffix-visible'], true)
  assert.equal(evidence['nametag-visibility-rule'], true)
  assert.equal(evidence['collision-rule'], true)
  assert.equal(evidence['persistence-after-reconnect'], true)
})
