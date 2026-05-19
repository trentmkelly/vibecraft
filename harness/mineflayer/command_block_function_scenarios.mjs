export const COMMAND_BLOCK_FUNCTION_SCENARIOS = {
  commandBlocks: [
    'redstone-pulse-executes',
    'source-stack-context',
    'last-output-captured',
    'sequence-mode',
    'auto-mode',
    'redstone-mode',
    'conditional-flag',
    'nbt-round-trip',
    'gui-read-write-sync',
    'facing-chain-order',
    'auto-redstone-parity'
  ],
  commandBlockMinecart: [
    'activator-rail-delay',
    'command-source-context',
    'nbt-round-trip',
    'last-output-track-output'
  ],
  functions: [
    'mcfunction-loading',
    'function-tags-loading',
    'tick-tag-invocation',
    'load-tag-invocation',
    'function-command-execution',
    'return-values',
    'macro-substitution',
    'function-argument-suggestions',
    'scheduled-functions',
    'permission-level',
    'nested-depth-limit',
    'tick-load-parity',
    'macro-parity',
    'depth-limit-parity'
  ],
  parseTree: [
    'root-literals',
    'argument-types',
    'redirects',
    'permission-gates',
    'signed-argument-metadata',
    'tab-completion-order'
  ]
}

export function createCommandBlockFunctionPlan(kind, options = {}) {
  const steps = COMMAND_BLOCK_FUNCTION_SCENARIOS[kind]
  if (!steps) throw new Error(`Unknown command block/function scenario ${kind}`)
  return {
    name: `mineflayer-command-block-function-${kebab(kind)}`,
    kind,
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'FunctionBot',
    steps,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false',
      'enable-command-block': 'true',
      'function-permission-level': String(options.functionPermissionLevel ?? 2)
    }
  }
}

export async function runCommandBlockFunctionScenario(kind, options = {}) {
  const plan = createCommandBlockFunctionPlan(kind, options)
  const evidence = { timeline: [] }
  const result = await (options.probe ?? commandBlockFunctionProbe)(plan, options)
  for (const step of plan.steps) {
    if (!result.steps?.[step]) throw new Error(`Missing command block/function evidence for ${step}`)
    recordCommandBlockFunction(evidence, step, result.details?.[step] ?? {})
  }
  return {
    plan,
    evidence,
    summary: summarizeCommandBlockFunction(evidence, plan)
  }
}

export function summarizeCommandBlockFunction(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'command_block_function')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordCommandBlockFunction(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'command_block_function', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function kebab(value) {
  return value.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)
}

async function commandBlockFunctionProbe() {
  throw new Error('commandBlockFunctionProbe requires a scenario-specific server fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const kind = process.argv[2] ?? 'commandBlocks'
  runCommandBlockFunctionScenario(kind).then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
