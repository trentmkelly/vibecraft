import { cp, mkdir, readFile, stat, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { normalizeArtifacts } from './runner.mjs'
import { formatParityDiff } from './assertions.mjs'

export async function writeLoginDebugBundle(options = {}) {
  const outputRoot = options.outputRoot ?? path.join(tmpdir(), 'vibecraft-mineflayer-debug')
  const name = options.name ?? `login-${Date.now()}`
  const bundleDir = path.join(outputRoot, sanitizeName(name))
  await mkdir(bundleDir, { recursive: true })

  const session = options.session ?? {}
  const profile = session.profile ?? {}
  const artifacts = session.collectArtifacts ? await session.collectArtifacts() : null
  const normalizedArtifacts = artifacts
    ? normalizeArtifacts(artifacts, { root: session.root, port: session.endpoint?.port })
    : null
  const parityDiff = options.parityDiff ?? options.parity?.diff ?? []

  await writeJson(path.join(bundleDir, 'manifest.json'), {
    name,
    createdAt: new Date().toISOString(),
    root: session.root,
    paths: session.paths,
    endpoint: session.endpoint,
    profile,
    uuid: session.uuid,
    error: session.error ? { message: session.error.message, stack: session.error.stack } : null,
    files: [
      'profile.json',
      'timeline.json',
      'packet-trace.json',
      'server.log',
      'normalized-artifacts.json',
      'parity-diff.json',
      'parity-diff.txt'
    ]
  })
  await writeJson(path.join(bundleDir, 'profile.json'), { profile, uuid: session.uuid })
  await writeJson(path.join(bundleDir, 'timeline.json'), session.timeline ?? [])
  await writeJson(path.join(bundleDir, 'packet-trace.json'), session.packetTrace ?? [])
  await writeFile(path.join(bundleDir, 'server.log'), formatLogs(session.serverLogs ?? []))
  await writeJson(path.join(bundleDir, 'normalized-artifacts.json'), normalizedArtifacts)
  await writeJson(path.join(bundleDir, 'parity-diff.json'), parityDiff)
  await writeFile(path.join(bundleDir, 'parity-diff.txt'), `${formatParityDiff(parityDiff)}\n`)

  if (options.includeWorld !== false && session.paths?.world && await exists(session.paths.world)) {
    await cp(session.paths.world, path.join(bundleDir, 'world'), { recursive: true })
  }
  if (session.paths?.serverProperties && await exists(session.paths.serverProperties)) {
    await cp(session.paths.serverProperties, path.join(bundleDir, 'server.properties'))
  }
  if (session.paths?.eula && await exists(session.paths.eula)) {
    await cp(session.paths.eula, path.join(bundleDir, 'eula.txt'))
  }

  return {
    bundleDir,
    manifest: path.join(bundleDir, 'manifest.json')
  }
}

export function formatLogs(logs) {
  return logs.map(entry => `[${entry.stream}] ${entry.text}`).join('')
}

function sanitizeName(name) {
  return String(name).replace(/[^A-Za-z0-9._-]+/g, '-')
}

async function writeJson(file, value) {
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`)
}

async function exists(file) {
  try {
    await stat(file)
    return true
  } catch {
    return false
  }
}
