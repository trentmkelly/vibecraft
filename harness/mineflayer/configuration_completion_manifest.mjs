export const configurationCompletionManifest = {
  protocolVersion: 775,
  finishConfigurationPacketId: 3,
  registryOrder: [
    'minecraft:worldgen/biome',
    'minecraft:chat_type',
    'minecraft:trim_pattern',
    'minecraft:trim_material',
    'minecraft:wolf_variant',
    'minecraft:wolf_sound_variant',
    'minecraft:pig_variant',
    'minecraft:pig_sound_variant',
    'minecraft:frog_variant',
    'minecraft:cat_variant',
    'minecraft:cat_sound_variant',
    'minecraft:cow_sound_variant',
    'minecraft:cow_variant',
    'minecraft:chicken_sound_variant',
    'minecraft:chicken_variant',
    'minecraft:zombie_nautilus_variant',
    'minecraft:painting_variant',
    'minecraft:dimension_type',
    'minecraft:damage_type',
    'minecraft:banner_pattern',
    'minecraft:jukebox_song',
    'minecraft:instrument'
  ],
  elementCounts: {
    'minecraft:banner_pattern': 43,
    'minecraft:worldgen/biome': 65,
    'minecraft:cat_sound_variant': 1,
    'minecraft:cat_variant': 11,
    'minecraft:chat_type': 7,
    'minecraft:chicken_sound_variant': 1,
    'minecraft:chicken_variant': 3,
    'minecraft:cow_sound_variant': 1,
    'minecraft:cow_variant': 3,
    'minecraft:damage_type': 50,
    'minecraft:dimension_type': 4,
    'minecraft:frog_variant': 3,
    'minecraft:instrument': 8,
    'minecraft:jukebox_song': 21,
    'minecraft:painting_variant': 1,
    'minecraft:pig_sound_variant': 1,
    'minecraft:pig_variant': 3,
    'minecraft:trim_material': 11,
    'minecraft:trim_pattern': 18,
    'minecraft:wolf_sound_variant': 1,
    'minecraft:wolf_variant': 9,
    'minecraft:zombie_nautilus_variant': 1
  },
  requiredElements: {
    'minecraft:banner_pattern': ['minecraft:bricks', 'minecraft:curly_border', 'minecraft:flower'],
    'minecraft:worldgen/biome': ['minecraft:end_barrens', 'minecraft:plains', 'minecraft:the_void'],
    'minecraft:cat_sound_variant': ['minecraft:default'],
    'minecraft:chat_type': [
      'minecraft:chat',
      'minecraft:emote_command',
      'minecraft:msg_command_incoming',
      'minecraft:msg_command_outgoing',
      'minecraft:say_command',
      'minecraft:team_msg_command_incoming',
      'minecraft:team_msg_command_outgoing'
    ],
    'minecraft:chicken_sound_variant': ['minecraft:default'],
    'minecraft:chicken_variant': ['minecraft:cold', 'minecraft:temperate', 'minecraft:warm'],
    'minecraft:cow_sound_variant': ['minecraft:default'],
    'minecraft:damage_type': ['minecraft:spear'],
    'minecraft:dimension_type': ['minecraft:overworld', 'minecraft:overworld_caves', 'minecraft:the_end', 'minecraft:the_nether'],
    'minecraft:instrument': ['minecraft:ponder_goat_horn'],
    'minecraft:jukebox_song': [
      'minecraft:11',
      'minecraft:13',
      'minecraft:5',
      'minecraft:creator',
      'minecraft:creator_music_box',
      'minecraft:lava_chicken',
      'minecraft:precipice',
      'minecraft:tears'
    ],
    'minecraft:pig_sound_variant': ['minecraft:default'],
    'minecraft:trim_material': [
      'minecraft:amethyst',
      'minecraft:copper',
      'minecraft:diamond',
      'minecraft:emerald',
      'minecraft:gold',
      'minecraft:iron',
      'minecraft:lapis',
      'minecraft:netherite',
      'minecraft:quartz',
      'minecraft:redstone',
      'minecraft:resin'
    ],
    'minecraft:wolf_sound_variant': ['minecraft:default'],
    'minecraft:zombie_nautilus_variant': ['minecraft:default']
  },
  requiredTags: {
    'minecraft:banner_pattern': {
      'minecraft:pattern_item/bordure_indented': [34],
      'minecraft:pattern_item/creeper': [36],
      'minecraft:pattern_item/field_masoned': [33],
      'minecraft:pattern_item/flower': [38],
      'minecraft:pattern_item/flow': [41],
      'minecraft:pattern_item/globe': [35],
      'minecraft:pattern_item/guster': [42],
      'minecraft:pattern_item/mojang': [39],
      'minecraft:pattern_item/piglin': [40],
      'minecraft:pattern_item/skull': [37]
    },
    'minecraft:damage_type': {
      'minecraft:bypasses_shield': [31, 22, 4, 6, 16, 18, 48, 5, 40, 10, 8, 17, 39, 27, 23, 32, 19, 36, 33, 2, 3, 7, 11, 13, 20, 21, 24, 25, 42],
      'minecraft:is_explosion': [15, 9, 35, 1],
      'minecraft:is_fire': [21, 3, 31, 24, 20, 46, 14]
    }
  },
  knownPacks: [
    { namespace: 'minecraft', id: 'core', version: '26.1.2' }
  ],
  playEntryPackets: [
    'clientbound/minecraft:login',
    'clientbound/minecraft:set_held_slot',
    'clientbound/minecraft:player_position',
    'clientbound/minecraft:chunk_batch_start',
    'clientbound/minecraft:level_chunk_with_light',
    'clientbound/minecraft:chunk_batch_finished'
  ]
}
