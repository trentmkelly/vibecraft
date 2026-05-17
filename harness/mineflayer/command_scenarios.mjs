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

function scenario(command, expectedFeedbackKey, options = {}) {
  return {
    command,
    expectedFeedbackKey,
    minPermission: options.minPermission ?? 0,
    runAsPermission: options.runAsPermission ?? options.minPermission ?? 0,
    target: options.target,
    expectDenied: options.expectDenied ?? false
  }
}
