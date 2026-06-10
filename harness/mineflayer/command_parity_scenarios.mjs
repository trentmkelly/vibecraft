export const COMMAND_PARITY_SCENARIOS = {
  executionTeleport: [
    'teleport-command-issued',
    'position-correction-observed',
    'success-feedback',
    'permission-failure'
  ],
  listLoginState: [
    'console-during-login',
    'bot-during-login',
    'after-join-count',
    'duplicate-replacement-count',
    'after-disconnect-count'
  ],
  lootCommand: [
    'loot-give',
    'loot-insert',
    'loot-spawn',
    'loot-replace',
    'block-source',
    'entity-source',
    'chest-source',
    'fishing-source',
    'custom-table-source'
  ],
  suggestions: [
    'root-command-tree',
    'argument-suggestions',
    'permission-filtering',
    'signed-command-metadata',
    'tab-completion-ordering'
  ],
  operatorSmoke: [
    'op',
    'deop',
    'whitelist',
    'ban',
    'pardon',
    'gamemode',
    'tp',
    'give',
    'effect',
    'feedback-and-permission-gates',
    'reconnect-visible-state'
  ],
  loginGated: [
    'list-after-readiness',
    'tell-after-readiness',
    'gamemode-after-readiness',
    'tp-after-readiness',
    'pre-readiness-blocked'
  ],
  permissionReload: [
    'ops-json-edited',
    'op-command-tree-delta',
    'deop-command-tree-delta',
    'reconnect-permission-state',
    'denied-feedback'
  ],
  beforeReady: [
    'login-boundary-command',
    'configuration-boundary-command',
    'play-transition-command',
    'rejection-or-queue-or-disconnect'
  ],
  resultConsistency: [
    'console-source',
    'op-bot-source',
    'non-op-bot-source',
    'command-block-source',
    'function-source',
    'success-count',
    'feedback-visibility',
    'observed-side-effects'
  ],
  parseTreeValidation: [
    'vanilla-command-dump',
    'vibecraft-command-dump',
    'parse-tree-diff',
    'scripted-execution-diff'
  ],
  chatCommand: [
    'signed-chat-fallback',
    'unsigned-chat-fallback',
    'system-messages',
    'command-feedback',
    'suggestions',
    'tab-completion'
  ]
}

export function createCommandParityPlan(kind, options = {}) {
  const steps = COMMAND_PARITY_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown command parity scenario ${kind}`)
  return {
    name: `mineflayer-command-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    primary: options.primary ?? 'CommandBot',
    secondary: options.secondary ?? 'CommandFriend',
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runCommandParityScenario(kind, options = {}) {
  const plan = createCommandParityPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? commandParityProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing command parity evidence for ${step}`)
    recordCommandParity(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizeCommandParity(evidence, plan)
  }
}

export function summarizeCommandParity(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'command_parity')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordCommandParity(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'command_parity', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function commandParityProbe() {
  throw new Error('commandParityProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'executionTeleport'
  runCommandParityScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
