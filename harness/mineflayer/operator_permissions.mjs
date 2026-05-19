export const OPERATOR_PERMISSION_STEPS = [
  'non-op-denied',
  'op-command-visible',
  'op-feedback-visible',
  'ops-json-reload-applied',
  'deop-feedback-visible',
  'tab-completion-updates'
]

export function createOperatorPermissionPlan(options = {}) {
  return {
    name: 'mineflayer-operator-permissions',
    mode: 'offline',
    auth: 'offline',
    nonOpUsername: options.nonOpUsername ?? 'PlainBot',
    opUsername: options.opUsername ?? 'OpBot',
    steps: OPERATOR_PERMISSION_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runOperatorPermissions(options = {}) {
  const plan = createOperatorPermissionPlan(options)
  const evidence = { timeline: [] }

  const denied = await (options.nonOpProbe ?? nonOpProbe)(plan, options)
  if (!denied.denied) throw new Error('Expected non-op permission denial')
  recordOperator(evidence, 'non-op-denied', denied)

  const visible = await (options.opVisibilityProbe ?? opVisibilityProbe)(plan, options)
  if (!visible.visible) throw new Error('Expected op-only command visibility for operator')
  recordOperator(evidence, 'op-command-visible', visible)

  const opFeedback = await (options.opCommandProbe ?? opCommandProbe)(plan, options)
  if (!opFeedback.ok || !/commands\.op\.success|opped/i.test(opFeedback.feedback ?? '')) {
    throw new Error(`Expected op feedback visibility, got ${opFeedback.feedback}`)
  }
  recordOperator(evidence, 'op-feedback-visible', opFeedback)

  const reload = await (options.reloadProbe ?? reloadProbe)(plan, options)
  if (!reload.reloaded) throw new Error('Expected ops.json reload to apply')
  recordOperator(evidence, 'ops-json-reload-applied', reload)

  const deopFeedback = await (options.deopCommandProbe ?? deopCommandProbe)(plan, options)
  if (!deopFeedback.ok || !/commands\.deop\.success|de-opped/i.test(deopFeedback.feedback ?? '')) {
    throw new Error(`Expected deop feedback visibility, got ${deopFeedback.feedback}`)
  }
  recordOperator(evidence, 'deop-feedback-visible', deopFeedback)

  const completion = await (options.tabCompletionProbe ?? tabCompletionProbe)(plan, options)
  if (!completion.updated) throw new Error('Expected tab completion to update after permission change')
  recordOperator(evidence, 'tab-completion-updates', completion)

  return {
    plan,
    evidence,
    summary: summarizeOperatorPermissions(evidence, plan)
  }
}

export function summarizeOperatorPermissions(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'operator_permission')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordOperator(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'operator_permission', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

async function nonOpProbe() {
  throw new Error('nonOpProbe requires a scenario-specific fixture')
}

async function opVisibilityProbe() {
  throw new Error('opVisibilityProbe requires a scenario-specific fixture')
}

async function opCommandProbe() {
  throw new Error('opCommandProbe requires a scenario-specific fixture')
}

async function reloadProbe() {
  throw new Error('reloadProbe requires a scenario-specific fixture')
}

async function deopCommandProbe() {
  throw new Error('deopCommandProbe requires a scenario-specific fixture')
}

async function tabCompletionProbe() {
  throw new Error('tabCompletionProbe requires a scenario-specific fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runOperatorPermissions().then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
