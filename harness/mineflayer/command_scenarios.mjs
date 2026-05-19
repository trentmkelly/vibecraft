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
