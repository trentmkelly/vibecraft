import { normalizeKickedMessage } from './assertions.mjs'

export function offlineChatScenarios(options = {}) {
  const sender = options.sender ?? 'VibeCraftBot'
  const target = options.target ?? 'VibeCraftBot1'
  return [
    chatScenario('publicChat', `${sender}: hello`, { sender }),
    chatScenario('privateMessage', `${sender} whispers to you: secret`, { sender, target }),
    chatScenario('formattedChat', '<VibeCraftBot> colored text', { contains: 'colored text' }),
    chatScenario('deathAnnouncement', `${sender} died`, { optional: true }),
    chatScenario('advancementAnnouncement', `${sender} has made the advancement`, { optional: true }),
    chatScenario('malformedDisconnect', 'multiplayer.disconnect.invalid_packet', {
      expectDisconnect: true
    })
  ]
}

export function observeChatScenario(session, scenario) {
  if (scenario.expectDisconnect) {
    const kicked = (session.timeline ?? []).find(event => event.name === 'kicked')
    const reason = normalizeKickedMessage(kicked?.summary?.[0])
    return {
      ok: Boolean(reason && reason.includes(scenario.expectedText)),
      reason
    }
  }
  const messages = (session.timeline ?? [])
    .filter(event => event.name === 'message')
    .flatMap(event => event.summary?.map(String) ?? [])
  const expected = scenario.contains ?? scenario.expectedText
  return {
    ok: scenario.optional || messages.some(message => message.includes(expected)),
    messages
  }
}

export function chatScenarioManifest(options = {}) {
  return {
    mode: 'offline',
    auth: 'offline',
    scenarios: offlineChatScenarios(options).map(scenario => ({
      name: scenario.name,
      expectedText: scenario.expectedText,
      optional: scenario.optional,
      expectDisconnect: scenario.expectDisconnect
    }))
  }
}

export function chatCommandScenario(options = {}) {
  const sender = options.sender ?? 'VibeCraftBot'
  const target = options.target ?? 'VibeCraftBot1'
  return {
    sender,
    target,
    steps: [
      'signed-chat-fallback',
      'unsigned-chat-fallback',
      'system-messages',
      'command-feedback',
      'suggestions',
      'tab-completion'
    ],
    commands: [
      `/tell ${target} secret`,
      '/me waves',
      '/help list'
    ]
  }
}

export function chatCommandScenarioManifest(options = {}) {
  const scenario = chatCommandScenario(options)
  return {
    mode: 'offline',
    auth: 'offline',
    sender: scenario.sender,
    target: scenario.target,
    commands: scenario.commands,
    steps: scenario.steps
  }
}

export function summarizeChatCommandEvidence(evidence, scenario = chatCommandScenario()) {
  const observations = new Map((evidence.timeline ?? [])
    .filter(event => event.name === 'chat_command')
    .map(event => [event.summary?.[0], event.summary?.[1] ?? {}]))
  const messages = (evidence.timeline ?? [])
    .filter(event => event.name === 'message')
    .flatMap(event => event.summary?.map(String) ?? [])
  const steps = {
    'signed-chat-fallback': observations.get('signed-chat-fallback')?.accepted === true,
    'unsigned-chat-fallback': observations.get('unsigned-chat-fallback')?.accepted === true,
    'system-messages': messages.some(message => message.includes('chat.type.system') ||
      message.includes('commands.message.display')),
    'command-feedback': messages.some(message => message.includes('commands.me.success') ||
      message.includes('commands.help.success')),
    suggestions: observations.get('suggestions')?.includesRoot === true,
    'tab-completion': observations.get('tab-completion')?.includesTarget === scenario.target
  }
  return { ok: Object.values(steps).every(Boolean), steps }
}

function chatScenario(name, expectedText, options = {}) {
  return {
    name,
    expectedText,
    sender: options.sender,
    target: options.target,
    contains: options.contains,
    optional: options.optional ?? false,
    expectDisconnect: options.expectDisconnect ?? false
  }
}
