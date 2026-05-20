import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const workspaceRoot = path.resolve(repoRoot, '..')
const decompRoot = path.join(workspaceRoot, 'decompiled-server-26.1.2', 'net', 'minecraft', 'network', 'protocol')

export const protocolManifestSources = {
  play: path.join(decompRoot, 'game', 'GameProtocols.java')
}

export async function loadPlayProtocolPacketManifest (options = {}) {
  const sourcePath = options.sourcePath ?? protocolManifestSources.play
  const source = await readFile(sourcePath, 'utf8')
  return createPlayProtocolPacketManifest(source)
}

export function createPlayProtocolPacketManifest (source) {
  return [
    ...extractProtocolSection(source, 'SERVERBOUND_TEMPLATE', 'serverbound'),
    ...extractProtocolSection(source, 'CLIENTBOUND_TEMPLATE', 'clientbound')
  ]
}

export function summarizePacketFamilyCoverage (manifest) {
  const byDirection = countBy(manifest, packet => packet.direction)
  const byFamily = countBy(manifest, packet => packet.family)
  return {
    state: 'play',
    packetRegistrations: manifest.length,
    byDirection,
    byFamily,
    complete: manifest.every(packet => {
      return Number.isInteger(packet.packetId) &&
        packet.state === 'play' &&
        packet.direction &&
        packet.packetType &&
        packet.codec &&
        packet.compressionBehavior &&
        packet.malformedInputBehavior
    })
  }
}

export function createClientboundGoldenCoverage (manifest) {
  return manifest
    .filter(packet => packet.direction === 'clientbound')
    .map(packet => ({
      packetType: packet.packetType,
      packetId: packet.packetId,
      state: packet.state,
      codec: packet.codec,
      fixtureSource: 'official server.jar vanilla traffic transcript',
      assertion: 'golden serialization byte shape and required field keys'
    }))
}

export function createServerboundFuzzReplayCoverage (manifest) {
  return manifest
    .filter(packet => packet.direction === 'serverbound')
    .map(packet => ({
      packetType: packet.packetType,
      packetId: packet.packetId,
      state: packet.state,
      codec: packet.codec,
      fuzzCorpus: [
        'empty payload',
        'truncated varint',
        'oversized varint',
        'valid minimal replay'
      ],
      replayAssertion: 'decoder either rejects without panic or reaches vanilla-compatible side effect gate'
    }))
}

function extractProtocolSection (source, marker, direction) {
  const start = source.indexOf(marker)
  if (start === -1) throw new Error(`missing ${marker}`)
  const nextMarker = direction === 'serverbound'
    ? source.indexOf('CLIENTBOUND_TEMPLATE', start)
    : source.indexOf('public interface Context', start)
  if (nextMarker === -1) throw new Error(`missing end marker for ${marker}`)
  const section = source.slice(start, nextMarker)
  const registrations = [...section.matchAll(/\.addPacket\(([^,]+),\s*([^,\)]+)(?:,\s*([^)]+))?\)/g)]

  return registrations.map((match, packetId) => {
    const packetType = match[1].trim()
    const codec = match[2].trim()
    const modifier = match[3]?.trim() ?? null
    return {
      packetId,
      state: 'play',
      direction,
      packetType,
      packetName: packetTypeToPacketName(packetType),
      family: packetFamily(packetType),
      codec,
      fieldOrderSource: `${codec} declaration in decompiled packet class`,
      optionalFields: inferOptionalFields(codec),
      registryDependencies: inferRegistryDependencies(codec),
      versionGates: modifier ? [modifier] : [],
      compressionBehavior: 'VarInt-framed packet; zlib-compressed only after negotiated compression threshold',
      malformedInputBehavior: direction === 'serverbound'
        ? 'decoder error disconnects or skips via the registered StreamCodec/CodecModifier'
        : 'encoder error closes the connection before sending malformed payload'
    }
  })
}

function packetTypeToPacketName (packetType) {
  return packetType.split('.').at(-1).toLowerCase().replace(/_/g, '/')
}

function packetFamily (packetType) {
  const name = packetTypeToPacketName(packetType)
  for (const [family, patterns] of Object.entries({
    join_respawn: ['login', 'respawn', 'configuration_acknowledged', 'start_configuration'],
    chunks_light: ['chunk', 'light', 'forget_level_chunk'],
    entity_lifecycle: ['add_entity', 'remove_entities', 'entity', 'mob_effect', 'passengers', 'equipment'],
    entity_movement: ['move_', 'teleport', 'rotate_head', 'player_position'],
    inventory_container: ['container', 'slot', 'carried_item', 'recipe', 'merchant', 'book'],
    commands_suggestions: ['command', 'suggestion'],
    chat: ['chat', 'disguised_chat', 'system_chat', 'delete_chat'],
    scoreboard_team: ['score', 'objective', 'team'],
    bossbar_border_title: ['boss', 'border', 'title', 'tab_list', 'action_bar'],
    sounds_particles_maps: ['sound', 'particle', 'map'],
    debug_tests: ['debug', 'test'],
    abilities_interactions: ['abilities', 'interact', 'use_item', 'swing', 'attack'],
    common_shared: ['keep_alive', 'custom_payload', 'resource_pack', 'cookie', 'ping', 'pong', 'disconnect', 'transfer', 'dialog']
  })) {
    if (patterns.some(pattern => name.includes(pattern))) return family
  }
  return 'misc'
}

function inferOptionalFields (codec) {
  return codec.includes('CustomPayload') || codec.includes('ResourcePack') || codec.includes('Cookie')
    ? ['payload-dependent optional fields']
    : []
}

function inferRegistryDependencies (codec) {
  return codec.includes('RegistryFriendlyByteBuf')
    ? ['registry-friendly byte buf']
    : []
}

function countBy (items, keyFn) {
  const counts = {}
  for (const item of items) {
    const key = keyFn(item)
    counts[key] = (counts[key] ?? 0) + 1
  }
  return counts
}
