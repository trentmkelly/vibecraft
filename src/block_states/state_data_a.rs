// GENERATED FILE - DO NOT EDIT. Regenerate with tools/generate_block_states.py
use super::StateProperty;

pub(super) const EMPTY: &[StateProperty] = &[];
const V0: &[&str] = &["true", "false"];
const V1: &[&str] = &["x", "y", "z"];
const V2: &[&str] = &["0", "1"];
const V3: &[&str] = &["0", "1", "2", "3", "4"];
const V4: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];
const V5: &[&str] = &["0", "1", "2", "3"];
const V6: &[&str] = &["1", "2", "3", "4", "5", "6", "7"];
const V7: &[&str] = &["north", "east", "south", "west", "up", "down"];
const V8: &[&str] = &[
    "harp",
    "basedrum",
    "snare",
    "hat",
    "bass",
    "flute",
    "bell",
    "guitar",
    "chime",
    "xylophone",
    "iron_xylophone",
    "cow_bell",
    "didgeridoo",
    "bit",
    "banjo",
    "pling",
    "trumpet",
    "trumpet_exposed",
    "trumpet_oxidized",
    "trumpet_weathered",
    "zombie",
    "skeleton",
    "creeper",
    "dragon",
    "wither_skeleton",
    "piglin",
    "custom_head",
];
const V9: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23", "24",
];
const V10: &[&str] = &["north", "south", "west", "east"];
const V11: &[&str] = &["head", "foot"];
const V12: &[&str] = &[
    "north_south",
    "east_west",
    "ascending_east",
    "ascending_west",
    "ascending_north",
    "ascending_south",
];
const V13: &[&str] = &["upper", "lower"];
const V14: &[&str] = &["normal", "sticky"];
const V15: &[&str] = &["unconnected", "right", "center", "left"];
const V16: &[&str] = &["uprooted", "dormant", "awake"];
const V17: &[&str] = &["top", "bottom"];
const V18: &[&str] = &[
    "straight",
    "inner_left",
    "inner_right",
    "outer_left",
    "outer_right",
];
const V19: &[&str] = &["single", "left", "right"];
const V20: &[&str] = &["up", "side", "none"];
const V21: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7"];
const V22: &[&str] = &["left", "right"];
const V23: &[&str] = &[
    "north_south",
    "east_west",
    "ascending_east",
    "ascending_west",
    "ascending_north",
    "ascending_south",
    "south_east",
    "south_west",
    "north_west",
    "north_east",
];
const V24: &[&str] = &["floor", "wall", "ceiling"];
const V25: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8"];
const V26: &[&str] = &["x", "z"];
const V27: &[&str] = &["0", "1", "2", "3", "4", "5", "6"];
const V28: &[&str] = &["1", "2", "3", "4"];
const V29: &[&str] = &["top", "bottom", "double"];
const V30: &[&str] = &["none", "low", "tall"];
const V31: &[&str] = &["1", "2", "3"];
const V32: &[&str] = &["0", "1", "2"];
const V33: &[&str] = &["compare", "subtract"];
const V34: &[&str] = &["down", "north", "south", "west", "east"];
const V35: &[&str] = &["0", "1", "2", "3", "4", "5"];
const V36: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23", "24", "25",
];
const V37: &[&str] = &["none", "small", "large"];
const V38: &[&str] = &["floor", "ceiling", "single_wall", "double_wall"];
const V39: &[&str] = &["save", "load", "corner", "data"];
const V40: &[&str] = &[
    "down_east",
    "down_north",
    "down_south",
    "down_west",
    "up_east",
    "up_north",
    "up_south",
    "up_west",
    "west_up",
    "east_up",
    "north_up",
    "south_up",
];
const V41: &[&str] = &["start", "log", "fail", "accept"];
const V42: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8"];
const V43: &[&str] = &["inactive", "active", "cooldown"];
const V44: &[&str] = &["standing", "sitting", "running", "star"];
const V45: &[&str] = &["tip_merge", "tip", "frustum", "middle", "base"];
const V46: &[&str] = &["up", "down"];
const V47: &[&str] = &["none", "unstable", "partial", "full"];
const V48: &[&str] = &[
    "inactive",
    "waiting_for_players",
    "active",
    "waiting_for_reward_ejection",
    "ejecting_reward",
    "cooldown",
];
const V49: &[&str] = &["inactive", "active", "unlocking", "ejecting"];
const P0: StateProperty = StateProperty {
    name: "snowy",
    values: V0,
};
const P1: StateProperty = StateProperty {
    name: "axis",
    values: V1,
};
const P2: StateProperty = StateProperty {
    name: "stage",
    values: V2,
};
const P3: StateProperty = StateProperty {
    name: "age",
    values: V3,
};
const P4: StateProperty = StateProperty {
    name: "hanging",
    values: V0,
};
const P5: StateProperty = StateProperty {
    name: "waterlogged",
    values: V0,
};
const P6: StateProperty = StateProperty {
    name: "level",
    values: V4,
};
const P7: StateProperty = StateProperty {
    name: "dusted",
    values: V5,
};
const P8: StateProperty = StateProperty {
    name: "distance",
    values: V6,
};
const P9: StateProperty = StateProperty {
    name: "persistent",
    values: V0,
};
const P10: StateProperty = StateProperty {
    name: "facing",
    values: V7,
};
const P11: StateProperty = StateProperty {
    name: "triggered",
    values: V0,
};
const P12: StateProperty = StateProperty {
    name: "instrument",
    values: V8,
};
const P13: StateProperty = StateProperty {
    name: "note",
    values: V9,
};
const P14: StateProperty = StateProperty {
    name: "powered",
    values: V0,
};
const P15: StateProperty = StateProperty {
    name: "facing",
    values: V10,
};
const P16: StateProperty = StateProperty {
    name: "occupied",
    values: V0,
};
const P17: StateProperty = StateProperty {
    name: "part",
    values: V11,
};
const P18: StateProperty = StateProperty {
    name: "shape",
    values: V12,
};
const P19: StateProperty = StateProperty {
    name: "extended",
    values: V0,
};
const P20: StateProperty = StateProperty {
    name: "half",
    values: V13,
};
const P21: StateProperty = StateProperty {
    name: "short",
    values: V0,
};
const P22: StateProperty = StateProperty {
    name: "type",
    values: V14,
};
const P23: StateProperty = StateProperty {
    name: "unstable",
    values: V0,
};
const P24: StateProperty = StateProperty {
    name: "slot_0_occupied",
    values: V0,
};
const P25: StateProperty = StateProperty {
    name: "slot_1_occupied",
    values: V0,
};
const P26: StateProperty = StateProperty {
    name: "slot_2_occupied",
    values: V0,
};
const P27: StateProperty = StateProperty {
    name: "slot_3_occupied",
    values: V0,
};
const P28: StateProperty = StateProperty {
    name: "slot_4_occupied",
    values: V0,
};
const P29: StateProperty = StateProperty {
    name: "slot_5_occupied",
    values: V0,
};
const P30: StateProperty = StateProperty {
    name: "side_chain",
    values: V15,
};
const P31: StateProperty = StateProperty {
    name: "age",
    values: V4,
};
const P32: StateProperty = StateProperty {
    name: "east",
    values: V0,
};
const P33: StateProperty = StateProperty {
    name: "north",
    values: V0,
};
const P34: StateProperty = StateProperty {
    name: "south",
    values: V0,
};
const P35: StateProperty = StateProperty {
    name: "up",
    values: V0,
};
const P36: StateProperty = StateProperty {
    name: "west",
    values: V0,
};
const P37: StateProperty = StateProperty {
    name: "creaking_heart_state",
    values: V16,
};
const P38: StateProperty = StateProperty {
    name: "natural",
    values: V0,
};
const P39: StateProperty = StateProperty {
    name: "half",
    values: V17,
};
const P40: StateProperty = StateProperty {
    name: "shape",
    values: V18,
};
const P41: StateProperty = StateProperty {
    name: "type",
    values: V19,
};
const P42: StateProperty = StateProperty {
    name: "east",
    values: V20,
};
const P43: StateProperty = StateProperty {
    name: "north",
    values: V20,
};
const P44: StateProperty = StateProperty {
    name: "power",
    values: V4,
};
const P45: StateProperty = StateProperty {
    name: "south",
    values: V20,
};
const P46: StateProperty = StateProperty {
    name: "west",
    values: V20,
};
const P47: StateProperty = StateProperty {
    name: "age",
    values: V21,
};
const P48: StateProperty = StateProperty {
    name: "moisture",
    values: V21,
};
const P49: StateProperty = StateProperty {
    name: "lit",
    values: V0,
};
const P50: StateProperty = StateProperty {
    name: "rotation",
    values: V4,
};
const P51: StateProperty = StateProperty {
    name: "hinge",
    values: V22,
};
const P52: StateProperty = StateProperty {
    name: "open",
    values: V0,
};
const P53: StateProperty = StateProperty {
    name: "shape",
    values: V23,
};
const P54: StateProperty = StateProperty {
    name: "attached",
    values: V0,
};
const P55: StateProperty = StateProperty {
    name: "face",
    values: V24,
};
const P56: StateProperty = StateProperty {
    name: "layers",
    values: V25,
};
const P57: StateProperty = StateProperty {
    name: "has_record",
    values: V0,
};
const P58: StateProperty = StateProperty {
    name: "axis",
    values: V26,
};
const P59: StateProperty = StateProperty {
    name: "bites",
    values: V27,
};
const P60: StateProperty = StateProperty {
    name: "delay",
    values: V28,
};
const P61: StateProperty = StateProperty {
    name: "locked",
    values: V0,
};
const P62: StateProperty = StateProperty {
    name: "down",
    values: V0,
};
const P63: StateProperty = StateProperty {
    name: "in_wall",
    values: V0,
};
const P64: StateProperty = StateProperty {
    name: "type",
    values: V29,
};
const P65: StateProperty = StateProperty {
    name: "east",
    values: V30,
};
const P66: StateProperty = StateProperty {
    name: "north",
    values: V30,
};
const P67: StateProperty = StateProperty {
    name: "south",
    values: V30,
};
const P68: StateProperty = StateProperty {
    name: "west",
    values: V30,
};
const P69: StateProperty = StateProperty {
    name: "age",
    values: V5,
};
const P70: StateProperty = StateProperty {
    name: "has_bottle_0",
    values: V0,
};
const P71: StateProperty = StateProperty {
    name: "has_bottle_1",
    values: V0,
};
const P72: StateProperty = StateProperty {
    name: "has_bottle_2",
    values: V0,
};
const P73: StateProperty = StateProperty {
    name: "level",
    values: V31,
};
const P74: StateProperty = StateProperty {
    name: "eye",
    values: V0,
};
const P75: StateProperty = StateProperty {
    name: "age",
    values: V32,
};
const P76: StateProperty = StateProperty {
    name: "disarmed",
    values: V0,
};
const P77: StateProperty = StateProperty {
    name: "conditional",
    values: V0,
};
const P78: StateProperty = StateProperty {
    name: "mode",
    values: V33,
};
const P79: StateProperty = StateProperty {
    name: "inverted",
    values: V0,
};
const P80: StateProperty = StateProperty {
    name: "enabled",
    values: V0,
};
const P81: StateProperty = StateProperty {
    name: "facing",
    values: V34,
};
const P82: StateProperty = StateProperty {
    name: "age",
    values: V35,
};
const P83: StateProperty = StateProperty {
    name: "age",
    values: V2,
};
const P84: StateProperty = StateProperty {
    name: "age",
    values: V36,
};
const P85: StateProperty = StateProperty {
    name: "eggs",
    values: V28,
};
const P86: StateProperty = StateProperty {
    name: "hatch",
    values: V32,
};
const P87: StateProperty = StateProperty {
    name: "hydration",
    values: V5,
};
const P88: StateProperty = StateProperty {
    name: "pickles",
    values: V28,
};
const P89: StateProperty = StateProperty {
    name: "leaves",
    values: V37,
};
const P90: StateProperty = StateProperty {
    name: "drag",
    values: V0,
};
const P91: StateProperty = StateProperty {
    name: "bottom",
    values: V0,
};
const P92: StateProperty = StateProperty {
    name: "distance",
    values: V21,
};
const P93: StateProperty = StateProperty {
    name: "has_book",
    values: V0,
};
const P94: StateProperty = StateProperty {
    name: "attachment",
    values: V38,
};
const P95: StateProperty = StateProperty {
    name: "signal_fire",
    values: V0,
};
const P96: StateProperty = StateProperty {
    name: "mode",
    values: V39,
};
const P97: StateProperty = StateProperty {
    name: "orientation",
    values: V40,
};
const P98: StateProperty = StateProperty {
    name: "mode",
    values: V41,
};
const P99: StateProperty = StateProperty {
    name: "level",
    values: V42,
};
const P100: StateProperty = StateProperty {
    name: "honey_level",
    values: V35,
};
const P101: StateProperty = StateProperty {
    name: "charges",
    values: V3,
};
const P102: StateProperty = StateProperty {
    name: "candles",
    values: V28,
};
const P103: StateProperty = StateProperty {
    name: "sculk_sensor_phase",
    values: V43,
};
const P104: StateProperty = StateProperty {
    name: "bloom",
    values: V0,
};
const P105: StateProperty = StateProperty {
    name: "can_summon",
    values: V0,
};
const P106: StateProperty = StateProperty {
    name: "shrieking",
    values: V0,
};
const P107: StateProperty = StateProperty {
    name: "copper_golem_pose",
    values: V44,
};
const P108: StateProperty = StateProperty {
    name: "thickness",
    values: V45,
};
const P109: StateProperty = StateProperty {
    name: "vertical_direction",
    values: V46,
};
const P110: StateProperty = StateProperty {
    name: "berries",
    values: V0,
};
const P111: StateProperty = StateProperty {
    name: "flower_amount",
    values: V28,
};
const P112: StateProperty = StateProperty {
    name: "segment_amount",
    values: V28,
};
const P113: StateProperty = StateProperty {
    name: "tilt",
    values: V47,
};
const P114: StateProperty = StateProperty {
    name: "cracked",
    values: V0,
};
const P115: StateProperty = StateProperty {
    name: "crafting",
    values: V0,
};
const P116: StateProperty = StateProperty {
    name: "ominous",
    values: V0,
};
const P117: StateProperty = StateProperty {
    name: "trial_spawner_state",
    values: V48,
};
const P118: StateProperty = StateProperty {
    name: "vault_state",
    values: V49,
};
const P119: StateProperty = StateProperty {
    name: "tip",
    values: V0,
};
pub(super) const L0: &[StateProperty] = &[P0];
pub(super) const L1: &[StateProperty] = &[P1];
pub(super) const L2: &[StateProperty] = &[P2];
pub(super) const L3: &[StateProperty] = &[P3, P4, P2, P5];
pub(super) const L4: &[StateProperty] = &[P6];
pub(super) const L5: &[StateProperty] = &[P7];
pub(super) const L6: &[StateProperty] = &[P5];
pub(super) const L7: &[StateProperty] = &[P8, P9, P5];
pub(super) const L8: &[StateProperty] = &[P10, P11];
pub(super) const L9: &[StateProperty] = &[P12, P13, P14];
pub(super) const L10: &[StateProperty] = &[P15, P16, P17];
pub(super) const L11: &[StateProperty] = &[P14, P18, P5];
pub(super) const L12: &[StateProperty] = &[P19, P10];
pub(super) const L13: &[StateProperty] = &[P20];
pub(super) const L14: &[StateProperty] = &[P10, P21, P22];
pub(super) const L15: &[StateProperty] = &[P10, P22];
pub(super) const L16: &[StateProperty] = &[P23];
pub(super) const L17: &[StateProperty] = &[P15, P24, P25, P26, P27, P28, P29];
pub(super) const L18: &[StateProperty] = &[P15, P14, P30, P5];
pub(super) const L19: &[StateProperty] = &[P15];
pub(super) const L20: &[StateProperty] = &[P31, P32, P33, P34, P35, P36];
pub(super) const L21: &[StateProperty] = &[P1, P37, P38];
pub(super) const L22: &[StateProperty] = &[P15, P39, P40, P5];
pub(super) const L23: &[StateProperty] = &[P15, P41, P5];
pub(super) const L24: &[StateProperty] = &[P42, P43, P44, P45, P46];
pub(super) const L25: &[StateProperty] = &[P47];
pub(super) const L26: &[StateProperty] = &[P48];
pub(super) const L27: &[StateProperty] = &[P15, P49];
pub(super) const L28: &[StateProperty] = &[P50, P5];
pub(super) const L29: &[StateProperty] = &[P15, P20, P51, P52, P14];
pub(super) const L30: &[StateProperty] = &[P15, P5];
pub(super) const L31: &[StateProperty] = &[P53, P5];
pub(super) const L32: &[StateProperty] = &[P54, P50, P5];
pub(super) const L33: &[StateProperty] = &[P55, P15, P14];
pub(super) const L34: &[StateProperty] = &[P14];
pub(super) const L35: &[StateProperty] = &[P49];
pub(super) const L36: &[StateProperty] = &[P56];
pub(super) const L37: &[StateProperty] = &[P31];
pub(super) const L38: &[StateProperty] = &[P57];
pub(super) const L39: &[StateProperty] = &[P32, P33, P34, P5, P36];
pub(super) const L40: &[StateProperty] = &[P58];
pub(super) const L41: &[StateProperty] = &[P59];
pub(super) const L42: &[StateProperty] = &[P60, P15, P61, P14];
pub(super) const L43: &[StateProperty] = &[P15, P39, P52, P14, P5];
pub(super) const L44: &[StateProperty] = &[P62, P32, P33, P34, P35, P36];
pub(super) const L45: &[StateProperty] = &[P1, P5];
pub(super) const L46: &[StateProperty] = &[P32, P33, P34, P35, P36];
pub(super) const L47: &[StateProperty] = &[P62, P32, P33, P34, P35, P5, P36];
pub(super) const L48: &[StateProperty] = &[P15, P63, P52, P14];
pub(super) const L49: &[StateProperty] = &[P64, P5];
pub(super) const L50: &[StateProperty] = &[P65, P66, P67, P35, P5, P68];
pub(super) const L51: &[StateProperty] = &[P69];
pub(super) const L52: &[StateProperty] = &[P70, P71, P72];
pub(super) const L53: &[StateProperty] = &[P73];
pub(super) const L54: &[StateProperty] = &[P74, P15];
pub(super) const L55: &[StateProperty] = &[P75, P15];
pub(super) const L56: &[StateProperty] = &[P54, P15, P14];
pub(super) const L57: &[StateProperty] = &[P54, P76, P32, P33, P14, P34, P36];
pub(super) const L58: &[StateProperty] = &[P77, P10];
pub(super) const L59: &[StateProperty] = &[P14, P50];
pub(super) const L60: &[StateProperty] = &[P15, P14];
pub(super) const L61: &[StateProperty] = &[P44];
pub(super) const L62: &[StateProperty] = &[P15, P78, P14];
pub(super) const L63: &[StateProperty] = &[P79, P44];
pub(super) const L64: &[StateProperty] = &[P80, P81];
pub(super) const L65: &[StateProperty] = &[P6, P5];
pub(super) const L66: &[StateProperty] = &[P50];
pub(super) const L67: &[StateProperty] = &[P10];
pub(super) const L68: &[StateProperty] = &[P82];
pub(super) const L69: &[StateProperty] = &[P83];
pub(super) const L70: &[StateProperty] = &[P3, P20];
pub(super) const L71: &[StateProperty] = &[P10, P14];
pub(super) const L72: &[StateProperty] = &[P84];
pub(super) const L73: &[StateProperty] = &[P85, P86];
pub(super) const L74: &[StateProperty] = &[P86];
pub(super) const L75: &[StateProperty] = &[P15, P87, P5];
pub(super) const L76: &[StateProperty] = &[P88, P5];
pub(super) const L77: &[StateProperty] = &[P83, P89, P2];
pub(super) const L78: &[StateProperty] = &[P90];
pub(super) const L79: &[StateProperty] = &[P91, P92, P5];
pub(super) const L80: &[StateProperty] = &[P10, P52];
pub(super) const L81: &[StateProperty] = &[P55, P15];
pub(super) const L82: &[StateProperty] = &[P15, P93, P14];
pub(super) const L83: &[StateProperty] = &[P94, P15, P14];
pub(super) const L84: &[StateProperty] = &[P4, P5];
pub(super) const L85: &[StateProperty] = &[P15, P49, P95, P5];
pub(super) const L86: &[StateProperty] = &[P96];
pub(super) const L87: &[StateProperty] = &[P97];
pub(super) const L88: &[StateProperty] = &[P98];
pub(super) const L89: &[StateProperty] = &[P99];
pub(super) const L90: &[StateProperty] = &[P15, P100];
pub(super) const L91: &[StateProperty] = &[P101];
pub(super) const L92: &[StateProperty] = &[P102, P49, P5];
pub(super) const L93: &[StateProperty] = &[P10, P5];
pub(super) const L94: &[StateProperty] = &[P44, P103, P5];
pub(super) const L95: &[StateProperty] = &[P15, P44, P103, P5];
pub(super) const L96: &[StateProperty] = &[P104];
pub(super) const L97: &[StateProperty] = &[P105, P106, P5];
pub(super) const L98: &[StateProperty] = &[P49, P14];
pub(super) const L99: &[StateProperty] = &[P107, P15, P5];
pub(super) const L100: &[StateProperty] = &[P10, P14, P5];
pub(super) const L101: &[StateProperty] = &[P108, P109, P5];
pub(super) const L102: &[StateProperty] = &[P84, P110];
pub(super) const L103: &[StateProperty] = &[P110];
pub(super) const L104: &[StateProperty] = &[P15, P111];
pub(super) const L105: &[StateProperty] = &[P15, P112];
pub(super) const L106: &[StateProperty] = &[P15, P113, P5];
pub(super) const L107: &[StateProperty] = &[P15, P20, P5];
pub(super) const L108: &[StateProperty] = &[P114, P15, P5];
pub(super) const L109: &[StateProperty] = &[P115, P97, P11];
pub(super) const L110: &[StateProperty] = &[P116, P117];
pub(super) const L111: &[StateProperty] = &[P15, P116, P118];
pub(super) const L112: &[StateProperty] = &[P91, P65, P66, P67, P68];
pub(super) const L113: &[StateProperty] = &[P119];
