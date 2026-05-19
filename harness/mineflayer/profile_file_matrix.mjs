import { offlineUuid } from './runner.mjs'

export const PROFILE_FILE_MATRIX_STEPS = [
  'fresh-profile',
  'returning-profile',
  'renamed-case-profile',
  'banned-profile',
  'whitelisted-profile',
  'operator-profile',
  'profile-files-use-offline-uuid'
]

export function createProfileFileMatrixPlan(options = {}) {
  return {
    name: 'mineflayer-profile-file-matrix',
    mode: 'offline',
    auth: 'offline',
    profiles: {
      fresh: options.fresh ?? 'ProfileFresh',
      renamedCase: options.renamedCase ?? 'profilefresh',
      banned: options.banned ?? 'ProfileBanned',
      whitelisted: options.whitelisted ?? 'ProfileListed',
      op: options.op ?? 'ProfileOp'
    },
    steps: PROFILE_FILE_MATRIX_STEPS,
    serverProperties: {
      'online-mode': 'false',
      'enforce-secure-profile': 'false'
    }
  }
}

export async function runProfileFileMatrix(options = {}) {
  const plan = createProfileFileMatrixPlan(options)
  const evidence = { timeline: [] }

  const fresh = await (options.joinProfile ?? joinProfile)(plan.profiles.fresh, 'fresh', options)
  assertProfile(fresh, plan.profiles.fresh)
  recordProfileFile(evidence, 'fresh-profile', profileEvidence(fresh))

  const returning = await (options.joinProfile ?? joinProfile)(plan.profiles.fresh, 'returning', options)
  assertProfile(returning, plan.profiles.fresh)
  assertSameUuid(fresh, returning)
  recordProfileFile(evidence, 'returning-profile', profileEvidence(returning))

  const renamed = await (options.joinProfile ?? joinProfile)(plan.profiles.renamedCase, 'renamed-case', options)
  assertProfile(renamed, plan.profiles.renamedCase)
  if (profileUuid(renamed) === profileUuid(fresh)) throw new Error('Case-renamed profile reused original-case UUID')
  recordProfileFile(evidence, 'renamed-case-profile', profileEvidence(renamed))

  const banned = await (options.accessProbe ?? accessProbe)(plan.profiles.banned, 'banned', options)
  if (!banned.rejected) throw new Error('Expected banned profile rejection')
  recordProfileFile(evidence, 'banned-profile', banned)

  const whitelisted = await (options.accessProbe ?? accessProbe)(plan.profiles.whitelisted, 'whitelisted', options)
  if (!whitelisted.allowed) throw new Error('Expected whitelisted profile to join')
  recordProfileFile(evidence, 'whitelisted-profile', whitelisted)

  const op = await (options.accessProbe ?? accessProbe)(plan.profiles.op, 'op', options)
  if (!op.allowed || !op.operator) throw new Error('Expected operator profile to join with op lookup')
  recordProfileFile(evidence, 'operator-profile', op)

  const files = await (options.profileFileProbe ?? profileFileProbe)(plan, [fresh, returning, renamed], options)
  verifyProfileFiles(plan, files)
  recordProfileFile(evidence, 'profile-files-use-offline-uuid', files)

  return {
    plan,
    evidence,
    summary: summarizeProfileFileMatrix(evidence, plan)
  }
}

export function summarizeProfileFileMatrix(evidence, plan) {
  const actions = (evidence.timeline ?? [])
    .filter(event => event.name === 'profile_file')
    .map(event => event.summary?.[0])
  const steps = Object.fromEntries(plan.steps.map(step => [step, actions.includes(step)]))
  return {
    ok: Object.values(steps).every(Boolean),
    steps
  }
}

export function recordProfileFile(evidence, step, details = {}) {
  evidence.timeline.push({ name: 'profile_file', at: Date.now(), summary: [step, details] })
  return { step, ...details }
}

function assertProfile(joined, username) {
  if (joined.profile?.username !== username) throw new Error(`Expected profile name ${username}, got ${joined.profile?.username}`)
  const expectedUuid = offlineUuid(username)
  if (profileUuid(joined) !== expectedUuid) throw new Error(`Expected UUID ${expectedUuid}, got ${profileUuid(joined)}`)
}

function assertSameUuid(left, right) {
  if (profileUuid(left) !== profileUuid(right)) throw new Error('Returning profile did not reuse the same UUID')
}

function profileUuid(joined) {
  return joined.profile?.actualUuid ?? joined.profile?.expectedUuid ?? joined.uuid
}

function profileEvidence(joined) {
  return {
    username: joined.profile?.username,
    uuid: profileUuid(joined)
  }
}

function verifyProfileFiles(plan, files) {
  const required = [plan.profiles.fresh, plan.profiles.renamedCase, plan.profiles.whitelisted, plan.profiles.op]
  for (const name of required) {
    const uuid = offlineUuid(name)
    const usercache = files.usercache?.find(entry => entry.name === name && entry.uuid === uuid)
    if (!usercache) throw new Error(`Missing usercache entry for ${name}`)
    if (!files.playerdata?.includes(`${uuid}.dat`)) throw new Error(`Missing playerdata file for ${name}`)
  }
  const op = files.ops?.find(entry => entry.name === plan.profiles.op && entry.uuid === offlineUuid(plan.profiles.op))
  if (!op) throw new Error('Missing ops.json offline UUID entry')
  const listed = files.whitelist?.find(entry => entry.name === plan.profiles.whitelisted && entry.uuid === offlineUuid(plan.profiles.whitelisted))
  if (!listed) throw new Error('Missing whitelist.json offline UUID entry')
  const banned = files.bannedPlayers?.find(entry => entry.name === plan.profiles.banned && entry.uuid === offlineUuid(plan.profiles.banned))
  if (!banned) throw new Error('Missing banned-players.json offline UUID entry')
}

async function joinProfile() {
  throw new Error('joinProfile requires a scenario-specific fixture')
}

async function accessProbe() {
  throw new Error('accessProbe requires a scenario-specific fixture')
}

async function profileFileProbe() {
  throw new Error('profileFileProbe requires a scenario-specific fixture')
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runProfileFileMatrix().then(result => {
    console.log(JSON.stringify({ plan: result.plan, summary: result.summary }, null, 2))
    process.exitCode = result.summary.ok ? 0 : 1
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
