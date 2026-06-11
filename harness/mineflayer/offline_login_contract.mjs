import { readFile } from 'node:fs/promises'
import { runObservedOfflineLogin } from './login_session.mjs'
import { offlineUuid } from './runner.mjs'

export async function runOfflineLoginContract(options = {}) {
  const session = await (options.runLogin ?? runObservedOfflineLogin)(options)
  const contract = await assertOfflineLoginContract(session, options)
  return {
    ok: contract.ok,
    session,
    contract,
    report: formatOfflineLoginContract(contract)
  }
}

export async function assertOfflineLoginContract(session, options = {}) {
  const checks = []
  const username = session.profile?.username ?? options.username ?? 'VibeCraftBot'
  const expectedUuid = options.expectedUuid ?? session.profile?.expectedUuid ?? offlineUuid(username)

  checks.push(check('serverReadiness', !session.error && Boolean(session.endpoint?.port), session.error?.message ?? `ready:${session.endpoint?.port}`))
  checks.push(check('eulaAccepted', await fileContains(session.paths?.eula, /^eula=true$/m), session.paths?.eula ?? '<missing eula path>'))
  checks.push(check(
    'offlineServerProperties',
    await fileContains(session.paths?.serverProperties, /^online-mode=false$/m),
    session.paths?.serverProperties ?? '<missing server.properties path>'
  ))
  checks.push(check(
    'deterministicProfile',
    session.profile?.expectedUuid === expectedUuid && (session.profile?.actualUuid == null || session.profile.actualUuid === expectedUuid),
    `${session.profile?.actualUuid ?? '<no actual uuid>'} expected ${expectedUuid}`
  ))
  checks.push(check(
    'playStateEntry',
    (session.timeline ?? []).some(event => event.name === 'spawn'),
    (session.timeline ?? []).map(event => event.name).join(',') || '<no events>'
  ))

  return {
    ok: checks.every(item => item.ok),
    username,
    expectedUuid,
    checks
  }
}

export function formatOfflineLoginContract(contract) {
  return contract.checks
    .map(item => `${item.ok ? 'PASS' : 'FAIL'} ${item.name}: ${item.detail}`)
    .join('\n')
}

async function fileContains(file, pattern) {
  if (!file) return false
  try {
    return pattern.test(await readFile(file, 'utf8'))
  } catch {
    return false
  }
}

function check(name, ok, detail) {
  return { name, ok, detail }
}
