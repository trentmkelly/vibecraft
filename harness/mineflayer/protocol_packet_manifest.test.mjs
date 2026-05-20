import assert from 'node:assert/strict'
import test from 'node:test'

import {
  createPlayProtocolPacketManifest,
  loadPlayProtocolPacketManifest,
  summarizePacketFamilyCoverage
} from './protocol_packet_manifest.mjs'

test('play protocol packet manifest extracts packet ids and codec metadata from GameProtocols', async () => {
  const manifest = await loadPlayProtocolPacketManifest()
  const summary = summarizePacketFamilyCoverage(manifest)

  assert.equal(summary.state, 'play')
  assert.equal(summary.packetRegistrations, 209)
  assert.deepEqual(summary.byDirection, { serverbound: 69, clientbound: 140 })
  assert.equal(summary.complete, true)
  assert.deepEqual(manifest[0], {
    packetId: 0,
    state: 'play',
    direction: 'serverbound',
    packetType: 'GamePacketTypes.SERVERBOUND_ACCEPT_TELEPORTATION',
    packetName: 'serverbound/accept/teleportation',
    family: 'entity_movement',
    codec: 'ServerboundAcceptTeleportationPacket.STREAM_CODEC',
    fieldOrderSource: 'ServerboundAcceptTeleportationPacket.STREAM_CODEC declaration in decompiled packet class',
    optionalFields: [],
    registryDependencies: [],
    versionGates: [],
    compressionBehavior: 'VarInt-framed packet; zlib-compressed only after negotiated compression threshold',
    malformedInputBehavior: 'decoder error disconnects or skips via the registered StreamCodec/CodecModifier'
  })
  assert.ok(manifest.some(packet => packet.packetType === 'GamePacketTypes.CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT' && packet.family === 'chunks_light'))
  assert.ok(manifest.some(packet => packet.packetType === 'GamePacketTypes.SERVERBOUND_SET_CREATIVE_MODE_SLOT' && packet.versionGates.includes('HAS_INFINITE_MATERIALS')))
  assert.ok(summary.byFamily.chunks_light > 0)
  assert.ok(summary.byFamily.inventory_container > 0)
  assert.ok(summary.byFamily.common_shared > 0)
})

test('play protocol packet manifest reports incomplete generated metadata', () => {
  const manifest = createPlayProtocolPacketManifest(`
    public static final UnboundProtocol SERVERBOUND_TEMPLATE = ProtocolInfoBuilder.contextServerboundProtocol(
      ConnectionProtocol.PLAY,
      builder -> builder.addPacket(GamePacketTypes.SERVERBOUND_KEEP_ALIVE, ServerboundKeepAlivePacket.STREAM_CODEC)
    );
    public static final SimpleUnboundProtocol CLIENTBOUND_TEMPLATE = ProtocolInfoBuilder.clientboundProtocol(
      ConnectionProtocol.PLAY,
      builder -> builder.addPacket(GamePacketTypes.CLIENTBOUND_KEEP_ALIVE, ClientboundKeepAlivePacket.STREAM_CODEC)
    );
    public interface Context {}
  `)
  manifest[0].codec = ''

  assert.equal(summarizePacketFamilyCoverage(manifest).complete, false)
})
