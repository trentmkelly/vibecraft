import { spawn } from 'node:child_process'
import { createHash } from 'node:crypto'
import { mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { readRegionFile, summarizeRegion } from './vanilla_region_reader.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const defaultServerJar = path.resolve(repoRoot, '..', 'server.jar')

export function chunkToRegionCoord (chunkCoord) {
  return Math.floor(chunkCoord / 32)
}

export function chunkToBlockCoord (chunkCoord) {
  return chunkCoord * 16
}

export function regionFileForChunk ({ x, z, dimension = 'overworld', levelName = 'world' }) {
  const region = `r.${chunkToRegionCoord(x)}.${chunkToRegionCoord(z)}.mca`
  switch (dimension) {
    case 'overworld':
      return path.join(levelName, 'dimensions', 'minecraft', 'overworld', 'region', region)
    case 'the_nether':
    case 'nether':
      return path.join(levelName, 'dimensions', 'minecraft', 'the_nether', 'region', region)
    case 'the_end':
    case 'end':
      return path.join(levelName, 'dimensions', 'minecraft', 'the_end', 'region', region)
    default:
      throw new Error(`unsupported dimension for region artifact: ${dimension}`)
  }
}

export function forceLoadCommandForChunk ({ x, z }) {
  return `forceload add ${chunkToBlockCoord(x)} ${chunkToBlockCoord(z)}`
}

export function dimensionForceLoadCommandForChunk ({ x, z, dimension = 'overworld' }) {
  const command = forceLoadCommandForChunk({ x, z })
  switch (dimension) {
    case 'overworld':
      return command
    case 'the_nether':
    case 'nether':
      return `execute in minecraft:the_nether run ${command}`
    case 'the_end':
    case 'end':
      return `execute in minecraft:the_end run ${command}`
    default:
      throw new Error(`unsupported dimension for force-load command: ${dimension}`)
  }
}

export function buildVanillaWorldgenOraclePlan ({
  chunks,
  seed = 8675309n,
  levelName = 'world',
  port = 0
}) {
  const normalizedChunks = chunks.map(chunk => ({
    x: Number(chunk.x),
    z: Number(chunk.z),
    dimension: chunk.dimension ?? 'overworld'
  }))
  const commands = [
    ...normalizedChunks
      .map(dimensionForceLoadCommandForChunk),
    'save-all flush',
    'stop'
  ]
  const regionFiles = [...new Set(normalizedChunks.map(chunk => regionFileForChunk({ ...chunk, levelName })))].sort()
  return {
    seed: seed.toString(),
    levelName,
    port,
    chunks: normalizedChunks,
    commands,
    regionFiles
  }
}

export async function writeVanillaServerFiles (root, {
  seed,
  levelName = 'world',
  port = 0,
  properties = {}
}) {
  await mkdir(root, { recursive: true })
  await writeFile(path.join(root, 'eula.txt'), 'eula=true\n')
  const merged = {
    'allow-flight': 'true',
    'enable-command-block': 'true',
    'level-name': levelName,
    'level-seed': seed.toString(),
    'max-tick-time': '0',
    'online-mode': 'false',
    'server-port': String(port),
    'spawn-protection': '0',
    'view-distance': '2',
    ...properties
  }
  const body = Object.entries(merged)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => `${key}=${value}`)
    .join('\n')
  await writeFile(path.join(root, 'server.properties'), `${body}\n`)
}

export async function runVanillaWorldgenOracle ({
  chunks,
  seed = 8675309n,
  root,
  serverJar = defaultServerJar,
  java = process.env.JAVA ?? 'java',
  levelName = 'world',
  port = 0,
  timeoutMs = 120_000
}) {
  const plan = buildVanillaWorldgenOraclePlan({ chunks, seed, levelName, port })
  await rm(root, { recursive: true, force: true })
  await writeVanillaServerFiles(root, plan)

  const child = spawn(java, ['-Xmx1G', '-jar', serverJar, '--nogui', '--universe', root, '--world', levelName, '--port', String(port)], {
    cwd: root,
    stdio: ['pipe', 'pipe', 'pipe']
  })
  let output = ''
  let sent = false

  const timer = setTimeout(() => {
    child.kill('SIGTERM')
  }, timeoutMs)

  child.stdout.on('data', chunk => {
    const text = chunk.toString('utf8')
    output += text
    if (!sent && /Done \([^)]+\)! For help, type "help"/.test(output)) {
      sent = true
      for (const command of plan.commands) {
        child.stdin.write(`${command}\n`)
      }
    }
  })
  child.stderr.on('data', chunk => {
    output += chunk.toString('utf8')
  })

  const exitCode = await new Promise((resolve, reject) => {
    child.once('error', reject)
    child.once('exit', code => resolve(code))
  }).finally(() => clearTimeout(timer))

  const artifacts = await collectRegionArtifacts(root, plan.regionFiles, plan.chunks)
  return {
    ok: exitCode === 0 && sent,
    exitCode,
    sentCommands: sent,
    plan,
    artifacts,
    logTail: output.split(/\r?\n/).slice(-80).join('\n')
  }
}

export async function collectRegionArtifacts (root, regionFiles, requestedChunks = []) {
  const requestedKeys = new Set(requestedChunks.map(chunk => `${chunk.x},${chunk.z}`))
  const artifacts = []
  for (const relativePath of regionFiles) {
    const absolutePath = path.join(root, relativePath)
    const data = await readFile(absolutePath)
    const region = summarizeRegion(await readRegionFile(absolutePath))
    const statusCounts = {}
    for (const chunk of region.chunks) {
      statusCounts[chunk.status ?? '<missing>'] = (statusCounts[chunk.status ?? '<missing>'] ?? 0) + 1
    }
    artifacts.push({
      path: relativePath,
      bytes: data.length,
      sha256: createHash('sha256').update(data).digest('hex'),
      chunkCount: region.chunkCount,
      statusCounts: Object.fromEntries(Object.entries(statusCounts).sort(([left], [right]) => left.localeCompare(right))),
      requestedChunks: region.chunks.filter(chunk => requestedKeys.has(`${chunk.chunkX},${chunk.chunkZ}`))
    })
  }
  return artifacts
}

export function buildVanillaWorldgenTraceReport (oracleResult) {
  const requestedChunks = (oracleResult.artifacts ?? [])
    .flatMap(artifact => (artifact.requestedChunks ?? []).map(chunk => traceChunk(artifact, chunk)))
    .sort((left, right) => left.dimension.localeCompare(right.dimension) || left.chunkX - right.chunkX || left.chunkZ - right.chunkZ)
  return {
    format: 'rustcraft-vanilla-worldgen-trace-v1',
    seed: oracleResult.plan?.seed,
    levelName: oracleResult.plan?.levelName,
    commandTrace: oracleResult.plan?.commands ?? [],
    regionArtifacts: (oracleResult.artifacts ?? []).map(artifact => ({
      path: artifact.path,
      bytes: artifact.bytes,
      sha256: artifact.sha256,
      chunkCount: artifact.chunkCount,
      statusCounts: artifact.statusCounts ?? {}
    })),
    requestedChunks
  }
}

function traceChunk (artifact, chunk) {
  return {
    dimension: dimensionFromRegionPath(artifact.path),
    chunkX: chunk.chunkX,
    chunkZ: chunk.chunkZ,
    finalStatus: chunk.status,
    heightmaps: chunk.heightmaps ?? {},
    biomePalette: chunk.biomePalette ?? [],
    sectionCount: chunk.sectionCount,
    nonEmptySectionCount: chunk.nonEmptySectionCount,
    sectionPalettes: (chunk.sections ?? []).map(section => ({
      y: section.y,
      blockPalette: section.blockPalette ?? [],
      blockStatesData: section.blockStatesData ?? null,
      biomePalette: section.biomePalette ?? [],
      biomeData: section.biomeData ?? null
    })),
    structures: chunk.structures ?? { startKeys: [], referenceKeys: [] },
    featureBlockSamples: featureBlockSamples(chunk),
    featureBlockPaletteCounts: featureBlockPaletteCounts(chunk),
    serializedChunkNbt: {
      payloadBytes: chunk.payloadBytes,
      payloadSha256: chunk.payloadSha256
    }
  }
}

function dimensionFromRegionPath (regionPath) {
  if (regionPath.includes('/the_nether/')) return 'the_nether'
  if (regionPath.includes('/the_end/')) return 'the_end'
  return 'overworld'
}

function featureBlockSamples (chunk) {
  return Object.keys(featureBlockPaletteCounts(chunk))
}

function featureBlockPaletteCounts (chunk) {
  const counts = {}
  for (const block of (chunk.sections ?? []).flatMap(section => section.blockPalette ?? [])) {
    if (!isFeatureLikeBlock(block)) continue
    counts[block] = (counts[block] ?? 0) + 1
  }
  for (const block of chunk.blockPalette ?? []) {
    if (!isFeatureLikeBlock(block) || counts[block] !== undefined) continue
    counts[block] = 0
  }
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => left.localeCompare(right)))
}

function isFeatureLikeBlock (block) {
  return (
    block.includes('ore') ||
    block.includes('log') ||
    block.includes('leaves') ||
    block.includes('grass') ||
    block.includes('flower') ||
    block.includes('mushroom') ||
    block.includes('vine') ||
    block.includes('coral') ||
    block.includes('kelp') ||
    block.includes('seagrass')
  )
}

export async function listRegionFiles (root, levelName = 'world') {
  const worldRoot = path.join(root, levelName)
  const regions = []
  for (const [dimension, relativeDir] of [
    ['overworld', path.join('dimensions', 'minecraft', 'overworld', 'region')],
    ['the_nether', path.join('dimensions', 'minecraft', 'the_nether', 'region')],
    ['the_end', path.join('dimensions', 'minecraft', 'the_end', 'region')]
  ]) {
    const dir = path.join(worldRoot, relativeDir)
    let entries = []
    try {
      entries = await readdir(dir)
    } catch {
      continue
    }
    for (const entry of entries.filter(name => name.endsWith('.mca')).sort()) {
      regions.push({ dimension, path: path.join(levelName, relativeDir, entry) })
    }
  }
  return regions
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const root = process.env.RUSTCRAFT_VANILLA_WORLDGEN_ROOT ?? path.join(repoRoot, 'target', 'vanilla-worldgen-oracle')
  const seed = BigInt(process.env.RUSTCRAFT_WORLDGEN_SEED ?? '8675309')
  const chunks = (process.env.RUSTCRAFT_WORLDGEN_CHUNKS ?? '0,0;1,0;-1,-1')
    .split(';')
    .filter(Boolean)
    .map(entry => {
      const [x, z, dimension] = entry.split(',')
      return { x: Number(x), z: Number(z), dimension: dimension ?? 'overworld' }
    })
  const result = await runVanillaWorldgenOracle({ root, seed, chunks })
  console.log(JSON.stringify(result, null, 2))
  process.exit(result.ok ? 0 : 1)
}
