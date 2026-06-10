import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import { spawn } from 'node:child_process'

const here = new URL('.', import.meta.url)
const defaultOfficialJar = path.resolve(new URL('../../../server.jar', here).pathname)
const defaultOfficialFixture = path.resolve(new URL('fixtures/official-26.1.2-configuration-transcript.json', here).pathname)

export function normalizeConfigurationTranscript (rawProbe) {
  return {
    protocolVersion: 775,
    enabledFeaturesPacketId: (rawProbe.config ?? []).find(packet => packet.id === 12)?.id ?? null,
    enabledFeatures: (rawProbe.config ?? []).find(packet => packet.id === 12)?.features ?? [],
    registries: (rawProbe.config ?? [])
      .filter(packet => packet.id === 7)
      .map(packet => ({
        registry: packet.registry,
        elements: packet.elements,
        elementIds: packet.elementIds,
        elementFieldPaths: packet.elementDataFields ?? {}
      })),
    tags: (rawProbe.config ?? [])
      .find(packet => packet.id === 13)?.registries
      ?.map(registry => ({
        registry: registry.registry,
        tags: registry.tags.map(tag => ({
          tag: tag.tag,
          entries: tag.entries
        }))
      })) ?? [],
    knownPacks: (rawProbe.config ?? [])
      .find(packet => packet.id === 14)?.packs ?? [],
    finishConfigurationPacketId: rawProbe.config?.at(-1)?.id,
    playPacketIds: (rawProbe.play ?? []).map(packet => packet.id)
  }
}

export async function writeOfficialConfigurationTranscriptFixture (options = {}) {
  const fixturePath = options.fixturePath ?? defaultOfficialFixture
  const transcript = options.transcript ?? await recordOfficialServerConfigurationTranscript(options)
  await mkdir(path.dirname(fixturePath), { recursive: true })
  await writeFile(fixturePath, `${JSON.stringify(transcript, null, 2)}\n`)
  return { fixturePath, transcript }
}

export function diffConfigurationTranscripts (actual, official) {
  const diffs = []
  compareSequence(diffs, 'registry order', actual.registries.map(entry => entry.registry), official.registries.map(entry => entry.registry))

  const officialRegistries = new Map(official.registries.map(entry => [entry.registry, entry]))
  for (const actualRegistry of actual.registries) {
    const officialRegistry = officialRegistries.get(actualRegistry.registry)
    if (!officialRegistry) continue
    if (actualRegistry.elements !== officialRegistry.elements) {
      diffs.push({
        path: `registry.${actualRegistry.registry}.elements`,
        actual: actualRegistry.elements,
        official: officialRegistry.elements
      })
    }
    compareSequence(
      diffs,
      `registry.${actualRegistry.registry}.elementIds`,
      actualRegistry.elementIds,
      officialRegistry.elementIds
    )
    for (const elementId of actualRegistry.elementIds ?? []) {
      if (!officialRegistry.elementIds?.includes(elementId)) continue
      compareSequence(
        diffs,
        `registry.${actualRegistry.registry}.element.${elementId}.fieldPaths`,
        actualRegistry.elementFieldPaths[elementId] ?? [],
        officialRegistry.elementFieldPaths[elementId] ?? []
      )
    }
  }

  const officialTags = new Map(official.tags.map(entry => [entry.registry, entry]))
  for (const actualTagRegistry of actual.tags) {
    const officialTagRegistry = officialTags.get(actualTagRegistry.registry)
    if (!officialTagRegistry) continue
    compareSequence(
      diffs,
      `tags.${actualTagRegistry.registry}.tagNames`,
      actualTagRegistry.tags.map(tag => tag.tag),
      officialTagRegistry.tags.map(tag => tag.tag)
    )
    const officialTagEntries = new Map(officialTagRegistry.tags.map(tag => [tag.tag, tag.entries]))
    for (const tag of actualTagRegistry.tags) {
      if (!officialTagEntries.has(tag.tag)) continue
      compareSequence(
        diffs,
        `tags.${actualTagRegistry.registry}.${tag.tag}.entries`,
        tag.entries,
        officialTagEntries.get(tag.tag)
      )
    }
  }

  compareSequence(diffs, 'knownPacks', actual.knownPacks.map(packKey), official.knownPacks.map(packKey))
  if (actual.finishConfigurationPacketId !== official.finishConfigurationPacketId) {
    diffs.push({
      path: 'finishConfigurationPacketId',
      actual: actual.finishConfigurationPacketId,
      official: official.finishConfigurationPacketId
    })
  }

  return {
    ok: diffs.length === 0,
    diffs
  }
}

export function omittedRegistriesFromTranscript (actual, official) {
  const actualRegistries = new Set(actual.registries.map(entry => entry.registry))
  return official.registries
    .map(entry => entry.registry)
    .filter(registry => !actualRegistries.has(registry))
}

export async function recordServerConfigurationTranscript (options = {}) {
  const env = {
    ...process.env,
    VIBECRAFT_RAW_PROBE_MODE: 'record',
    VIBECRAFT_HOST: options.host ?? '127.0.0.1',
    VIBECRAFT_PORT: String(options.port ?? 25565),
    VIBECRAFT_USERNAME: options.username ?? 'TranscriptProbe'
  }

  const probe = await runProcess(process.execPath, ['raw_26_1_2_join_probe.mjs'], {
    cwd: new URL('.', import.meta.url),
    env,
    timeoutMs: options.timeoutMs ?? 45_000
  })

  if (probe.code !== 0) {
    throw new Error(`raw probe failed: ${probe.stderr || probe.stdout}`)
  }

  return normalizeConfigurationTranscript(JSON.parse(probe.stdout))
}

export async function recordOfficialServerConfigurationTranscript (options = {}) {
  const port = options.port ?? 25566
  const workdir = await mkdtemp(path.join(os.tmpdir(), 'vibecraft-vanilla-transcript-'))
  let server
  try {
    await writeFile(path.join(workdir, 'eula.txt'), 'eula=true\n')
    await writeFile(path.join(workdir, 'server.properties'), [
      'online-mode=false',
      'enforce-secure-profile=false',
      'enable-status=true',
      'network-compression-threshold=-1',
      `server-port=${port}`,
      'level-name=world',
      ''
    ].join('\n'))

    server = await startOfficialServer({
      jar: options.jar ?? defaultOfficialJar,
      cwd: workdir,
      timeoutMs: options.startTimeoutMs ?? 120_000
    })

    return await recordServerConfigurationTranscript({
      host: '127.0.0.1',
      port,
      username: options.username ?? 'OfficialProbe',
      timeoutMs: options.probeTimeoutMs ?? 60_000
    })
  } finally {
    if (server) await stopOfficialServer(server)
    if (!options.keepArtifacts) await rm(workdir, { recursive: true, force: true })
  }
}

async function startOfficialServer ({ jar, cwd, timeoutMs }) {
  const child = spawn('java', ['-Xmx1G', '-Xms1G', '-jar', jar, 'nogui'], {
    cwd,
    stdio: ['pipe', 'pipe', 'pipe']
  })
  let output = ''

  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      child.kill('SIGTERM')
      reject(new Error(`official server did not become ready within ${timeoutMs}ms:\n${output}`))
    }, timeoutMs)

    const onData = chunk => {
      output += chunk.toString()
      if (output.includes('Done (')) {
        clearTimeout(timeout)
        resolve()
      }
    }

    child.stdout.on('data', onData)
    child.stderr.on('data', onData)
    child.once('exit', code => {
      clearTimeout(timeout)
      reject(new Error(`official server exited before readiness with code ${code}:\n${output}`))
    })
  })

  return child
}

async function stopOfficialServer (child) {
  if (child.exitCode !== null) return
  child.stdin.write('stop\n')
  await new Promise(resolve => {
    const timeout = setTimeout(() => {
      child.kill('SIGTERM')
      resolve()
    }, 15_000)
    child.once('exit', () => {
      clearTimeout(timeout)
      resolve()
    })
  })
}

async function runProcess (command, args, options) {
  return await new Promise(resolve => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    let stdout = ''
    let stderr = ''
    const timeout = setTimeout(() => child.kill('SIGTERM'), options.timeoutMs)
    child.stdout.on('data', chunk => { stdout += chunk })
    child.stderr.on('data', chunk => { stderr += chunk })
    child.on('close', code => {
      clearTimeout(timeout)
      resolve({ code, stdout, stderr })
    })
  })
}

function compareSequence (diffs, path, actual, official) {
  if (actual.length === official.length && actual.every((value, index) => value === official[index])) return
  diffs.push({ path, actual, official })
}

function packKey (pack) {
  return `${pack.namespace}:${pack.id}:${pack.version}`
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const mode = process.argv[2] ?? 'vibecraft'
  const transcript = mode === 'official'
    ? await recordOfficialServerConfigurationTranscript({
        port: Number(process.env.VANILLA_TRANSCRIPT_PORT ?? 25566),
        jar: process.env.OFFICIAL_SERVER_JAR,
        keepArtifacts: process.env.VIBECRAFT_KEEP_ARTIFACTS === '1'
      })
    : await recordServerConfigurationTranscript({
        host: process.env.VIBECRAFT_HOST,
        port: Number(process.env.VIBECRAFT_PORT ?? 25565)
      })
  console.log(JSON.stringify(transcript, null, 2))
}
