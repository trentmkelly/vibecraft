import { readdir, readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import { normalizeLoginArtifacts, normalizeLoginDiff } from './artifact_normalizer.mjs'

export async function collectOfflineLoginArtifact(session, options = {}) {
  const role = options.role ?? 'vibecraft'
  const root = session.root
  const world = session.paths?.world ?? path.join(root, 'world')
  const serverPropertiesPath = session.paths?.serverProperties ?? path.join(root, 'server.properties')
  const eulaPath = session.paths?.eula ?? path.join(root, 'eula.txt')
  const usercachePath = path.join(root, 'usercache.json')
  const playerdataDir = path.join(world, 'playerdata')
  const playerdataPaths = await listPlayerdataPaths(playerdataDir, root)

  return {
    role,
    root,
    endpoint: session.endpoint,
    profile: session.profile,
    uuid: session.uuid ?? session.profile?.actualUuid ?? session.profile?.expectedUuid,
    botUuid: session.profile?.actualUuid ?? session.uuid ?? session.profile?.expectedUuid,
    expectedUuid: session.profile?.expectedUuid,
    files: {
      serverProperties: await readOptional(serverPropertiesPath),
      eula: await readOptional(eulaPath),
      usercache: await readJsonOptional(usercachePath),
      playerdataPaths
    },
    logs: session.serverLogs ?? [],
    normalized: normalizeLoginArtifacts({
      logs: session.serverLogs ?? [],
      serverProperties: await readOptional(serverPropertiesPath),
      eula: await readOptional(eulaPath),
      usercache: await readJsonOptional(usercachePath),
      playerdataPaths,
      uuid: session.uuid ?? session.profile?.actualUuid ?? session.profile?.expectedUuid
    }, {
      roots: [root],
      ports: [session.endpoint?.port].filter(port => port != null),
      usernames: [session.profile?.username].filter(Boolean)
    })
  }
}

export async function collectOfflineLoginArtifactPair(officialSession, vibecraftSession) {
  const official = await collectOfflineLoginArtifact(officialSession, { role: 'official' })
  const vibecraft = await collectOfflineLoginArtifact(vibecraftSession, { role: 'vibecraft' })
  return {
    official,
    vibecraft,
    normalizedLogDiff: diffNormalizedLogs(official, vibecraft)
  }
}

export function diffNormalizedLogs(official, vibecraft) {
  return normalizeLoginDiff(compareArrays(
    official.normalized.logs,
    vibecraft.normalized.logs,
    'logs'
  ), {
    roots: [official.root, vibecraft.root],
    ports: [official.endpoint?.port, vibecraft.endpoint?.port].filter(port => port != null),
    usernames: [official.profile?.username, vibecraft.profile?.username].filter(Boolean)
  })
}

async function listPlayerdataPaths(playerdataDir, root) {
  try {
    const entries = await readdir(playerdataDir)
    return entries
      .filter(entry => entry.endsWith('.dat'))
      .sort()
      .map(entry => path.relative(root, path.join(playerdataDir, entry)))
  } catch {
    return []
  }
}

async function readOptional(file) {
  try {
    return await readFile(file, 'utf8')
  } catch {
    return null
  }
}

async function readJsonOptional(file) {
  if (!await exists(file)) return null
  return JSON.parse(await readFile(file, 'utf8'))
}

async function exists(file) {
  try {
    await stat(file)
    return true
  } catch {
    return false
  }
}

function compareArrays(left, right, pathName) {
  return JSON.stringify(left) === JSON.stringify(right)
    ? []
    : [{ path: pathName, official: left, rebuilt: right }]
}
