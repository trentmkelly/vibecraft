export const GAME_STATE_SCENARIOS = {
  gamerules: [
    'toggle-keepInventory-and-die',
    'toggle-doImmediateRespawn-and-die',
    'toggle-sendCommandFeedback-and-run-command',
    'toggle-doDaylightCycle-and-observe-time',
    'toggle-mobGriefing-and-trigger-griefing-mob',
    'client-observable-behavior-diffed-against-vanilla'
  ],
  statsAdvancements: [
    'movement-stat-action',
    'mining-stat-action',
    'crafting-stat-action',
    'death-stat-action',
    'recipe-unlock-action',
    'client-stats-packet-observed',
    'client-advancement-packet-observed',
    'saved-stats-json-after-reconnect',
    'saved-advancements-json-after-reconnect'
  ],
  scoreboardObjectives: [
    'objective-create',
    'score-update',
    'display-sidebar',
    'display-list',
    'display-below-name',
    'display-hide',
    'objective-remove',
    'persistence-after-reconnect'
  ],
  scoreboardTeams: [
    'sidebar-display-visible',
    'list-display-visible',
    'below-name-display-visible',
    'team-color-visible',
    'team-prefix-visible',
    'team-suffix-visible',
    'nametag-visibility-rule',
    'collision-rule',
    'persistence-after-reconnect'
  ]
}

export function createGameStateScenarioPlan() {
  return {
    name: 'mineflayer-game-state-scenarios',
    mode: 'offline',
    auth: 'offline',
    scenarios: Object.entries(GAME_STATE_SCENARIOS).map(([name, required]) => ({
      name,
      required
    }))
  }
}

export function summarizeGameStateEvidence(evidence, plan = createGameStateScenarioPlan()) {
  const scenarios = plan.scenarios.map(scenario => {
    const observed = evidence[scenario.name] ?? {}
    const missing = scenario.required.filter(key => !observed[key])
    return {
      name: scenario.name,
      ok: missing.length === 0,
      missing
    }
  })
  return {
    ok: scenarios.every(result => result.ok),
    scenarios
  }
}

export function buildGameruleEvidence(observations) {
  return {
    'toggle-keepInventory-and-die': observations.keepInventory?.inventoryRetained === true,
    'toggle-doImmediateRespawn-and-die': observations.doImmediateRespawn?.deathScreenSkipped === true,
    'toggle-sendCommandFeedback-and-run-command': observations.sendCommandFeedback?.feedbackSuppressed === true,
    'toggle-doDaylightCycle-and-observe-time': observations.doDaylightCycle?.timeStopped === true,
    'toggle-mobGriefing-and-trigger-griefing-mob': observations.mobGriefing?.worldUnchanged === true,
    'client-observable-behavior-diffed-against-vanilla': observations.vanillaDiff?.ok === true
  }
}

export function buildStatsAdvancementsEvidence(observations) {
  return {
    'movement-stat-action': hasAction(observations, 'movement'),
    'mining-stat-action': hasAction(observations, 'mining'),
    'crafting-stat-action': hasAction(observations, 'crafting'),
    'death-stat-action': hasAction(observations, 'death'),
    'recipe-unlock-action': hasAction(observations, 'recipe-unlock'),
    'client-stats-packet-observed': hasPacket(observations, 'award_stats'),
    'client-advancement-packet-observed': hasPacket(observations, 'update_advancements'),
    'saved-stats-json-after-reconnect': observations.savedFiles?.statsJsonAfterReconnect === true,
    'saved-advancements-json-after-reconnect': observations.savedFiles?.advancementsJsonAfterReconnect === true
  }
}

export function buildScoreboardObjectiveEvidence(observations) {
  return {
    'objective-create': hasCommand(observations, 'scoreboard objectives add'),
    'score-update': hasCommand(observations, 'scoreboard players set'),
    'display-sidebar': hasDisplay(observations, 'sidebar'),
    'display-list': hasDisplay(observations, 'list'),
    'display-below-name': hasDisplay(observations, 'below_name'),
    'display-hide': hasCommand(observations, 'scoreboard objectives setdisplay sidebar'),
    'objective-remove': hasCommand(observations, 'scoreboard objectives remove'),
    'persistence-after-reconnect': observations.persistence?.objectivesAfterReconnect === true
  }
}

export function buildScoreboardTeamEvidence(observations) {
  return {
    'sidebar-display-visible': hasDisplay(observations, 'sidebar'),
    'list-display-visible': hasDisplay(observations, 'list'),
    'below-name-display-visible': hasDisplay(observations, 'below_name'),
    'team-color-visible': observations.team?.colorVisible === true,
    'team-prefix-visible': observations.team?.prefixVisible === true,
    'team-suffix-visible': observations.team?.suffixVisible === true,
    'nametag-visibility-rule': observations.team?.nametagVisibilityMatchesVanilla === true,
    'collision-rule': observations.team?.collisionMatchesVanilla === true,
    'persistence-after-reconnect': observations.persistence?.teamsAfterReconnect === true
  }
}

function hasAction(observations, action) {
  return observations.actions?.includes(action) === true
}

function hasPacket(observations, packetName) {
  return observations.packets?.some(packet => packet.name === packetName) === true
}

function hasCommand(observations, prefix) {
  return observations.commands?.some(command => command.startsWith(prefix)) === true
}

function hasDisplay(observations, slot) {
  return observations.displays?.includes(slot) === true
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log(JSON.stringify(createGameStateScenarioPlan(), null, 2))
}
