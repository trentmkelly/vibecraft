import { normalizeKickedMessage } from './assertions.mjs'

export function offlineChatScenarios(options = {}) {
  const sender = options.sender ?? 'RustCraftBot'
  const target = options.target ?? 'RustCraftBot1'
  return [
    chatScenario('publicChat', `${sender}: hello`, { sender }),
    chatScenario('privateMessage', `${sender} whispers to you: secret`, { sender, target }),
    chatScenario('formattedChat', '<RustCraftBot> colored text', { contains: 'colored text' }),
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
