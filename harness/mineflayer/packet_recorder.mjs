export const PROTOCOL_STATES = ['status', 'login', 'configuration', 'play']

export function createPacketRecorder(options = {}) {
  const packets = []
  const stateFilter = new Set(options.states ?? PROTOCOL_STATES)
  return {
    packets,
    record(packet) {
      const normalized = normalizePacket(packet)
      if (!stateFilter.has(normalized.state)) return null
      packets.push(normalized)
      return normalized
    },
    attachClient(client, attachOptions = {}) {
      const direction = attachOptions.direction ?? 'inbound'
      const packetHandler = (data, meta = {}) => {
        this.record({
          direction,
          state: meta.state ?? attachOptions.state ?? 'play',
          name: meta.name ?? 'unknown',
          data
        })
      }
      const rawHandler = (buffer, meta = {}) => {
        if (attachOptions.raw !== true) return
        this.record({
          direction: `${direction}:raw`,
          state: meta.state ?? attachOptions.state ?? 'play',
          name: meta.name ?? 'raw',
          data: { length: buffer?.length ?? 0 }
        })
      }
      client.on('packet', packetHandler)
      client.on('raw', rawHandler)
      return () => {
        client.off?.('packet', packetHandler) ?? client.removeListener('packet', packetHandler)
        client.off?.('raw', rawHandler) ?? client.removeListener('raw', rawHandler)
      }
    },
    snapshot(filter = {}) {
      return filterPackets(packets, filter).map(packet => ({ ...packet, data: cloneJson(packet.data) }))
    },
    clear() {
      packets.length = 0
    }
  }
}

export async function replayPackets(packets, target, options = {}) {
  const selected = filterPackets(packets, options)
  for (const packet of selected) {
    if (options.mode === 'emit') {
      target.emit('packet', cloneJson(packet.data), { name: packet.name, state: packet.state })
    } else {
      target.write(packet.name, cloneJson(packet.data))
    }
    if (options.delayMs) await delay(options.delayMs)
  }
  return selected.length
}

export function packetFlowSummary(packets) {
  return packets.map(packet => ({
    direction: packet.direction,
    state: packet.state,
    name: packet.name,
    keys: Object.keys(packet.data ?? {}).sort()
  }))
}

export function normalizePacket(packet) {
  return {
    direction: packet.direction ?? 'inbound',
    state: packet.state ?? 'play',
    name: packet.name ?? 'unknown',
    at: packet.at ?? Date.now(),
    data: normalizePacketData(packet.data)
  }
}

function filterPackets(packets, filter = {}) {
  const states = filter.states ? new Set(filter.states) : null
  const names = filter.names ? new Set(filter.names) : null
  const directions = filter.directions ? new Set(filter.directions) : null
  return packets.filter(packet =>
    (!states || states.has(packet.state)) &&
    (!names || names.has(packet.name)) &&
    (!directions || directions.has(packet.direction))
  )
}

function normalizePacketData(data) {
  if (data == null) return null
  if (Buffer.isBuffer(data)) return { hex: data.toString('hex') }
  if (Array.isArray(data)) return data.map(normalizePacketData)
  if (typeof data !== 'object') return data
  return Object.fromEntries(
    Object.entries(data)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, value]) => [key, normalizePacketData(value)])
  )
}

function cloneJson(value) {
  return value == null ? value : JSON.parse(JSON.stringify(value))
}

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
