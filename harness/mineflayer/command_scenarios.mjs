import { issueCommand } from './bot_actions.mjs'

export function offlineCommandScenarios(options = {}) {
  const primary = options.primary ?? 'RustCraftBot'
  const secondary = options.secondary ?? 'RustCraftBot1'
  return [
    scenario('/list', 'commands.list.players', { minPermission: 0 }),
    scenario(`/tell ${secondary} hello`, 'commands.message.display', { minPermission: 0, target: secondary }),
    scenario(`/msg ${secondary} hello`, 'commands.message.display', { minPermission: 0, target: secondary }),
    scenario('/me waves', 'commands.me.success', { minPermission: 0 }),
    scenario('/help list', 'commands.help.success', { minPermission: 0 }),
    scenario('/seed', 'commands.seed.success', { minPermission: 2 }),
    scenario(`/gamemode creative ${primary}`, 'commands.gamemode.success.other', {
      minPermission: 2,
      target: primary
    }),
    scenario('/gamemode creative', 'commands.generic.permission', {
      minPermission: 2,
      runAsPermission: 0,
      expectDenied: true
    })
  ]
}

export async function runCommandScenario(session, commandScenario, options = {}) {
  const dispatch = options.dispatch ?? issueCommand
  const action = dispatch(session, commandScenario.command)
  const observation = observeCommandFeedback(session, commandScenario)
  return {
    scenario: commandScenario,
    action,
    observation,
    ok: commandScenario.expectDenied ? observation.denied : observation.matched
  }
}

export function observeCommandFeedback(session, commandScenario) {
  const messages = (session.timeline ?? []).filter(event => event.name === 'message')
  const matched = messages.some(event =>
    event.summary?.some(part => String(part).includes(commandScenario.expectedFeedbackKey))
  )
  const denied = messages.some(event =>
    event.summary?.some(part => /permission|commands\.generic\.permission/i.test(String(part)))
  )
  return {
    matched,
    denied,
    messages: messages.map(event => event.summary)
  }
}

export function commandScenarioManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    commands: offlineCommandScenarios(options).map(entry => ({
      command: entry.command,
      expectedFeedbackKey: entry.expectedFeedbackKey,
      minPermission: entry.minPermission,
      expectDenied: entry.expectDenied
    }))
  }
}

export function operatorCommandSmokeScenarios(options = {}) {
  const primary = options.primary ?? 'OpBot'
  const secondary = options.secondary ?? 'PlainBot'
  return [
    scenario(`/op ${secondary}`, 'commands.op.success', { minPermission: 3, reconnectVisibleState: true }),
    scenario(`/deop ${secondary}`, 'commands.deop.success', { minPermission: 3, reconnectVisibleState: true }),
    scenario(`/whitelist add ${secondary}`, 'commands.whitelist.add.success', { minPermission: 3, reconnectVisibleState: true }),
    scenario(`/ban ${secondary}`, 'commands.ban.success', { minPermission: 3, reconnectVisibleState: true }),
    scenario(`/pardon ${secondary}`, 'commands.pardon.success', { minPermission: 3, reconnectVisibleState: true }),
    scenario(`/gamemode creative ${primary}`, 'commands.gamemode.success.other', { minPermission: 2, target: primary }),
    scenario(`/tp ${primary} 4 70 4`, 'commands.teleport.success.location.single', { minPermission: 2, target: primary }),
    scenario(`/give ${primary} minecraft:stone 2`, 'commands.give.success.single', { minPermission: 2, target: primary }),
    scenario(`/effect give ${primary} minecraft:speed 5 1`, 'commands.effect.give.success.single', { minPermission: 2, target: primary })
  ]
}

export function operatorCommandSmokeManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    source: 'op-bot',
    commands: operatorCommandSmokeScenarios(options).map(entry => ({
      command: entry.command,
      expectedFeedbackKey: entry.expectedFeedbackKey,
      minPermission: entry.minPermission,
      target: entry.target,
      reconnectVisibleState: entry.reconnectVisibleState ?? false
    }))
  }
}

export function listLoginStateScenarios(options = {}) {
  const primary = options.primary ?? 'ListBot'
  const duplicate = options.duplicate ?? primary
  return [
    listScenario('console-during-login', 'console', 'during-login', []),
    listScenario('bot-during-login', 'bot', 'during-login', []),
    listScenario('after-join', 'console', 'after-join', [primary]),
    listScenario('after-duplicate-replacement', 'console', 'after-duplicate-replacement', [duplicate]),
    listScenario('after-disconnect', 'console', 'after-disconnect', [])
  ]
}

export function listLoginStateManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    command: '/list',
    scenarios: listLoginStateScenarios(options).map(entry => ({
      name: entry.name,
      source: entry.source,
      phase: entry.phase,
      expectedPlayerCount: entry.expectedNames.length,
      expectedNames: entry.expectedNames
    }))
  }
}

export function teleportCommandExecutionScenario(options = {}) {
  const username = options.username ?? 'TeleportBot'
  const position = options.position ?? { x: 8, y: 72, z: -6 }
  return {
    command: `/tp ${username} ${position.x} ${position.y} ${position.z}`,
    username,
    position,
    expectedFeedbackKey: 'commands.teleport.success.location.single',
    permissionDeniedKey: 'commands.generic.permission',
    requiredSteps: [
      'teleport-command-issued',
      'position-correction-observed',
      'success-feedback',
      'non-op-permission-failure'
    ]
  }
}

export function summarizeTeleportCommandEvidence(evidence, scenario = teleportCommandExecutionScenario()) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'teleport_command')
    .map(event => event.summary?.[0])
  const messages = (evidence.timeline ?? [])
    .filter(event => event.name === 'message')
    .flatMap(event => event.summary?.map(String) ?? [])
  const position = (evidence.timeline ?? [])
    .find(event => event.name === 'position' && event.summary?.[0] === scenario.username)
    ?.summary?.[1]
  const steps = {
    'teleport-command-issued': actions.includes('issued'),
    'position-correction-observed': Boolean(position) &&
      position.x === scenario.position.x &&
      position.y === scenario.position.y &&
      position.z === scenario.position.z,
    'success-feedback': messages.some(message => message.includes(scenario.expectedFeedbackKey)),
    'non-op-permission-failure': messages.some(message => message.includes(scenario.permissionDeniedKey))
  }
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function loginGatedCommandScenarios(options = {}) {
  const primary = options.primary ?? 'ReadyBot'
  const secondary = options.secondary ?? 'FriendBot'
  return [
    gatedScenario('/list', 'commands.list.players', { minPermission: 0 }),
    gatedScenario(`/tell ${secondary} ready`, 'commands.message.display', { minPermission: 0, target: secondary }),
    gatedScenario(`/gamemode creative ${primary}`, 'commands.gamemode.success.other', { minPermission: 2, target: primary }),
    gatedScenario(`/tp ${primary} 0 80 0`, 'commands.teleport.success.location.single', { minPermission: 2, target: primary })
  ]
}

export function loginGatedCommandManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    readinessBoundary: 'play-state-ready',
    commands: loginGatedCommandScenarios(options).map(entry => ({
      command: entry.command,
      expectedFeedbackKey: entry.expectedFeedbackKey,
      minPermission: entry.minPermission,
      target: entry.target,
      preReadyBlocked: true
    }))
  }
}

export function summarizeLoginGatedCommandEvidence(evidence, scenarios = loginGatedCommandScenarios()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'login_gated_command')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const steps = Object.fromEntries(scenarios.map(scenario => {
    const observed = observations.get(scenario.command)
    const ok = observed?.preReadyBlocked === true &&
      observed?.postReadyRan === true &&
      String(observed?.feedback ?? '').includes(scenario.expectedFeedbackKey)
    return [scenario.command, ok]
  }))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function commandPermissionReloadScenario(options = {}) {
  const username = options.username ?? 'ReloadBot'
  return {
    username,
    steps: [
      'ops-json-edited',
      'op-command-tree-delta',
      'deop-command-tree-delta',
      'reconnect-permission-state',
      'denied-feedback'
    ],
    expectedDeniedFeedbackKey: 'commands.generic.permission'
  }
}

export function commandPermissionReloadManifest(options = {}) {
  const scenario = commandPermissionReloadScenario(options)
  return {
    mode: 'offline',
    auth: 'offline',
    username: scenario.username,
    files: ['ops.json'],
    commands: ['/op', '/deop'],
    steps: scenario.steps
  }
}

export function summarizeCommandPermissionReloadEvidence(evidence, scenario = commandPermissionReloadScenario()) {
  const events = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'command_permission_reload')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const deniedFeedback = (evidence.timeline ?? [])
    .filter(event => event.name === 'message')
    .flatMap(event => event.summary?.map(String) ?? [])
    .some(message => message.includes(scenario.expectedDeniedFeedbackKey))
  const steps = {
    'ops-json-edited': events.get('ops-json-edited')?.written === true,
    'op-command-tree-delta': events.get('op-command-tree-delta')?.visible === true,
    'deop-command-tree-delta': events.get('deop-command-tree-delta')?.visible === false,
    'reconnect-permission-state': events.get('reconnect-permission-state')?.persisted === true,
    'denied-feedback': deniedFeedback
  }
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function commandBeforeReadyScenarios(options = {}) {
  const primary = options.primary ?? 'EarlyBot'
  return [
    beforeReadyScenario('login', `/tell ${primary} early`, 'reject'),
    beforeReadyScenario('configuration', '/list', 'reject'),
    beforeReadyScenario('configuration', `/tp ${primary} 0 80 0`, 'disconnect'),
    beforeReadyScenario('play-before-loaded', '/help list', 'queue-or-reject')
  ]
}

export function commandBeforeReadyManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    readinessBoundary: 'play-state-ready',
    scenarios: commandBeforeReadyScenarios(options).map(entry => ({
      phase: entry.phase,
      command: entry.command,
      expectedBehavior: entry.expectedBehavior
    }))
  }
}

export function summarizeCommandBeforeReadyEvidence(evidence, scenarios = commandBeforeReadyScenarios()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'command_before_ready')
    .map(event => [`${event.summary?.[0]}:${event.summary?.[1]}`, event.summary?.[2] ?? {}]))
  const steps = Object.fromEntries(scenarios.map(scenario => {
    const observed = observations.get(`${scenario.phase}:${scenario.command}`)
    const ok = observed?.beforeReady === true &&
      observed?.postReadyAccepted === true &&
      behaviorMatches(observed?.behavior, scenario.expectedBehavior)
    return [`${scenario.phase}:${scenario.command}`, ok]
  }))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function commandResultConsistencyScenarios(options = {}) {
  const primary = options.primary ?? 'ResultBot'
  return [
    resultScenario('console', `/gamemode creative ${primary}`, 'commands.gamemode.success.self', {
      successCount: 1,
      sideEffect: 'gamemode-creative'
    }),
    resultScenario('op-bot', '/list', 'commands.list.players', {
      minSuccessCount: 1,
      feedbackVisibleToSource: true
    }),
    resultScenario('non-op-bot', `/gamemode survival ${primary}`, 'commands.generic.permission', {
      successCount: 0,
      denied: true
    }),
    resultScenario('command-block', `/say ${primary}`, 'commands.say.success', {
      successCount: 1,
      feedbackVisibleToOperators: true
    }),
    resultScenario('function', `/tp ${primary} 0 80 0`, 'commands.teleport.success.location.single', {
      successCount: 1,
      sideEffect: 'position-correction'
    })
  ]
}

export function commandResultConsistencyManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    scenarios: commandResultConsistencyScenarios(options).map(entry => ({
      source: entry.source,
      command: entry.command,
      expectedFeedbackKey: entry.expectedFeedbackKey,
      successCount: entry.successCount,
      minSuccessCount: entry.minSuccessCount,
      denied: entry.denied,
      sideEffect: entry.sideEffect,
      feedbackVisibleToSource: entry.feedbackVisibleToSource,
      feedbackVisibleToOperators: entry.feedbackVisibleToOperators
    }))
  }
}

export function summarizeCommandResultConsistencyEvidence(evidence, scenarios = commandResultConsistencyScenarios()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'command_result_consistency')
    .map(event => [`${event.summary?.[0]}:${event.summary?.[1]}`, event.summary?.[2] ?? {}]))
  const steps = Object.fromEntries(scenarios.map(scenario => {
    const observed = observations.get(`${scenario.source}:${scenario.command}`)
    const successCountOk = scenario.minSuccessCount === undefined
      ? observed?.successCount === scenario.successCount
      : observed?.successCount >= scenario.minSuccessCount
    const feedbackOk = String(observed?.feedback ?? '').includes(scenario.expectedFeedbackKey)
    const sideEffectOk = !scenario.sideEffect || (observed?.sideEffects ?? []).includes(scenario.sideEffect)
    const deniedOk = !scenario.denied || observed?.denied === true
    const visibilityOk = scenario.feedbackVisibleToSource !== true || observed?.feedbackVisibleToSource === true
    const operatorVisibilityOk = scenario.feedbackVisibleToOperators !== true ||
      observed?.feedbackVisibleToOperators === true
    return [`${scenario.source}:${scenario.command}`, Boolean(successCountOk &&
      feedbackOk &&
      sideEffectOk &&
      deniedOk &&
      visibilityOk &&
      operatorVisibilityOk)]
  }))
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function commandSuggestionScenario(options = {}) {
  return {
    permissionLevels: options.permissionLevels ?? [0, 2, 3, 4],
    steps: [
      'root-command-tree',
      'argument-suggestions',
      'permission-filtering',
      'signed-command-metadata',
      'tab-completion-ordering'
    ],
    probes: [
      '',
      'gamemode ',
      'tell ',
      'execute '
    ]
  }
}

export function commandSuggestionManifest(options = {}) {
  const scenario = commandSuggestionScenario(options)
  return {
    mode: 'offline',
    auth: 'offline',
    comparedAgainst: 'official-server.jar',
    permissionLevels: scenario.permissionLevels,
    probes: scenario.probes,
    steps: scenario.steps
  }
}

export function summarizeCommandSuggestionEvidence(evidence, scenario = commandSuggestionScenario()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'command_suggestion')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const root = observations.get('root-command-tree')
  const args = observations.get('argument-suggestions')
  const permissions = observations.get('permission-filtering')
  const signed = observations.get('signed-command-metadata')
  const ordering = observations.get('tab-completion-ordering')
  const steps = {
    'root-command-tree': root?.matchesVanilla === true,
    'argument-suggestions': scenario.probes.every(probe => (args?.probes ?? []).includes(probe)),
    'permission-filtering': scenario.permissionLevels.every(level =>
      (permissions?.levels ?? []).includes(level)),
    'signed-command-metadata': signed?.matchesVanilla === true,
    'tab-completion-ordering': ordering?.stableVanillaOrder === true
  }
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function lootCommandScenario(options = {}) {
  const target = options.target ?? 'LootBot'
  return {
    target,
    commands: [
      `/loot give ${target} loot minecraft:chests/simple_dungeon`,
      '/loot insert 0 64 0 loot minecraft:chests/simple_dungeon',
      '/loot spawn 0 64 0 fish minecraft:gameplay/fishing 0 64 0',
      `/loot replace entity ${target} container.0 loot minecraft:chests/simple_dungeon`
    ],
    sources: [
      'block-source',
      'entity-source',
      'chest-source',
      'fishing-source',
      'custom-table-source'
    ],
    effects: [
      'inventory-update',
      'window-update',
      'dropped-item-entity'
    ]
  }
}

export function lootCommandManifest(options = {}) {
  const scenario = lootCommandScenario(options)
  return {
    mode: 'offline',
    auth: 'offline',
    comparedAgainst: 'official-server.jar',
    target: scenario.target,
    commands: scenario.commands,
    sources: scenario.sources,
    effects: scenario.effects
  }
}

export function summarizeLootCommandEvidence(evidence, scenario = lootCommandScenario()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'loot_command')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const commands = observations.get('commands')?.commands ?? []
  const sources = observations.get('sources')?.sources ?? []
  const effects = observations.get('effects')?.effects ?? []
  const steps = {
    commands: scenario.commands.every(command => commands.includes(command)),
    sources: scenario.sources.every(source => sources.includes(source)),
    effects: scenario.effects.every(effect => effects.includes(effect)),
    vanilla: observations.get('vanilla-comparison')?.matches === true
  }
  return { ok: Object.values(steps).every(Boolean), steps }
}

export function summarizeListLoginStateEvidence(evidence, scenarios = listLoginStateScenarios()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'list_command')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const steps = Object.fromEntries(scenarios.map(scenario => {
    const observed = observations.get(scenario.name)
    const names = observed?.names ?? []
    const count = observed?.count ?? names.length
    const ok = count === scenario.expectedNames.length &&
      scenario.expectedNames.every(name => names.includes(name))
    return [scenario.name, ok]
  }))
  return { ok: Object.values(steps).every(Boolean), steps }
}

function listScenario(name, source, phase, expectedNames) {
  return { name, source, phase, expectedNames }
}

function beforeReadyScenario(phase, command, expectedBehavior) {
  return { phase, command, expectedBehavior }
}

function resultScenario(source, command, expectedFeedbackKey, options = {}) {
  return {
    source,
    command,
    expectedFeedbackKey,
    successCount: options.successCount,
    minSuccessCount: options.minSuccessCount,
    denied: options.denied ?? false,
    sideEffect: options.sideEffect,
    feedbackVisibleToSource: options.feedbackVisibleToSource ?? false,
    feedbackVisibleToOperators: options.feedbackVisibleToOperators ?? false
  }
}

function behaviorMatches(observed, expected) {
  if (expected === 'queue-or-reject') {
    return observed === 'queued' || observed === 'reject'
  }
  return observed === expected
}

function gatedScenario(command, expectedFeedbackKey, options = {}) {
  return {
    command,
    expectedFeedbackKey,
    minPermission: options.minPermission ?? 0,
    target: options.target
  }
}

function scenario(command, expectedFeedbackKey, options = {}) {
  return {
    command,
    expectedFeedbackKey,
    minPermission: options.minPermission ?? 0,
    runAsPermission: options.runAsPermission ?? options.minPermission ?? 0,
    target: options.target,
    reconnectVisibleState: options.reconnectVisibleState,
    expectDenied: options.expectDenied ?? false
  }
}
