use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::core_orientation::{
    CoreAxisModel, CoreDirectionModel, PositionModel, Vec3PositionModel,
};
use crate::registry::Identifier;

pub const CORE_PACKAGE_NULL_MARKED: bool = true;

pub fn core_package_null_marked() -> bool {
    CORE_PACKAGE_NULL_MARKED
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientTextureAssetModel {
    DownloadedTexture {
        texture_path: Identifier,
        url: String,
    },
    ResourceTexture {
        id: Identifier,
        texture_path: Identifier,
    },
}

impl ClientTextureAssetModel {
    pub fn downloaded_texture(texture_path: Identifier, url: impl Into<String>) -> Self {
        Self::DownloadedTexture {
            texture_path,
            url: url.into(),
        }
    }

    pub fn resource_texture(id: Identifier, texture_path: Identifier) -> Self {
        Self::ResourceTexture { id, texture_path }
    }

    pub fn resource_texture_from_id(id: Identifier) -> Result<Self, String> {
        let texture_path = Identifier::new(id.namespace(), &format!("textures/{}.png", id.path()))?;
        Ok(Self::ResourceTexture { id, texture_path })
    }

    pub fn id(&self) -> &Identifier {
        match self {
            Self::DownloadedTexture { texture_path, .. } => texture_path,
            Self::ResourceTexture { id, .. } => id,
        }
    }

    pub fn texture_path(&self) -> &Identifier {
        match self {
            Self::DownloadedTexture { texture_path, .. }
            | Self::ResourceTexture { texture_path, .. } => texture_path,
        }
    }

    pub fn default_field_name() -> &'static str {
        "asset_id"
    }

    pub fn stream_value(&self) -> String {
        self.id().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClonerModel {
    id: String,
    fail_encode: bool,
    fail_decode: bool,
}

impl ClonerModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fail_encode: false,
            fail_decode: false,
        }
    }

    pub fn failing_encode(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fail_encode: true,
            fail_decode: false,
        }
    }

    pub fn failing_decode(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fail_encode: false,
            fail_decode: true,
        }
    }

    pub fn clone_value(&self, value: &str, from: &str, to: &str) -> Result<String, String> {
        if self.fail_encode {
            return Err(format!("Failed to encode: {} encode refused", self.id));
        }
        let encoded = format!("{}:{}:{}", self.id, from, value);
        if self.fail_decode {
            return Err(format!("Failed to decode: {} decode refused", self.id));
        }
        let prefix = format!("{}:{}:", self.id, from);
        let payload = encoded
            .strip_prefix(&prefix)
            .ok_or_else(|| format!("Failed to decode: invalid payload {encoded}"))?;
        Ok(format!("{to}:{payload}"))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ClonerFactoryModel {
    cloners: BTreeMap<String, ClonerModel>,
}

impl ClonerFactoryModel {
    pub fn add_codec(&mut self, key: impl Into<String>, cloner: ClonerModel) -> &mut Self {
        self.cloners.insert(key.into(), cloner);
        self
    }

    pub fn cloner(&self, key: &str) -> Option<&ClonerModel> {
        self.cloners.get(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor3DTypeModel {
    Inside,
    Face,
    Edge,
    Corner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor3DModel {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    width: i32,
    height: i32,
    depth: i32,
    end: i32,
    index: i32,
    next_x: i32,
    next_y: i32,
    next_z: i32,
}

impl Cursor3DModel {
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let depth = max_z - min_z + 1;
        Self {
            origin_x: min_x,
            origin_y: min_y,
            origin_z: min_z,
            width,
            height,
            depth,
            end: width * height * depth,
            index: 0,
            next_x: 0,
            next_y: 0,
            next_z: 0,
        }
    }

    pub fn advance(&mut self) -> bool {
        if self.index == self.end {
            return false;
        }
        self.next_x = self.index % self.width;
        let slice = self.index / self.width;
        self.next_y = slice % self.height;
        self.next_z = slice / self.height;
        self.index += 1;
        true
    }

    pub fn next_x(&self) -> i32 {
        self.origin_x + self.next_x
    }

    pub fn next_y(&self) -> i32 {
        self.origin_y + self.next_y
    }

    pub fn next_z(&self) -> i32 {
        self.origin_z + self.next_z
    }

    pub fn next_type(&self) -> Cursor3DTypeModel {
        match self.boundary_axis_count() {
            0 => Cursor3DTypeModel::Inside,
            1 => Cursor3DTypeModel::Face,
            2 => Cursor3DTypeModel::Edge,
            _ => Cursor3DTypeModel::Corner,
        }
    }

    fn boundary_axis_count(&self) -> usize {
        [
            (self.next_x, self.width),
            (self.next_y, self.height),
            (self.next_z, self.depth),
        ]
        .into_iter()
        .filter(|(coordinate, size)| *coordinate == 0 || *coordinate == *size - 1)
        .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonNullListModel<T> {
    values: Vec<T>,
    default_value: Option<T>,
}

impl<T: Clone> NonNullListModel<T> {
    pub fn create() -> Self {
        Self {
            values: Vec::new(),
            default_value: None,
        }
    }

    pub fn create_with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            default_value: None,
        }
    }

    pub fn with_size(size: usize, default_value: Option<T>) -> Result<Self, String> {
        let default_value = default_value.ok_or_else(|| "defaultValue".to_string())?;
        Ok(Self {
            values: vec![default_value.clone(); size],
            default_value: Some(default_value),
        })
    }

    pub fn of(default_value: Option<T>, values: Vec<T>) -> Self {
        Self {
            values,
            default_value,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> &T {
        &self.values[index]
    }

    pub fn set(&mut self, index: usize, value: Option<T>) -> Result<T, String> {
        let value = value.ok_or_else(|| "element".to_string())?;
        Ok(std::mem::replace(&mut self.values[index], value))
    }

    pub fn add(&mut self, index: usize, value: Option<T>) -> Result<(), String> {
        let value = value.ok_or_else(|| "element".to_string())?;
        self.values.insert(index, value);
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.values.remove(index)
    }

    pub fn clear(&mut self) {
        if let Some(default_value) = &self.default_value {
            self.values.fill(default_value.clone());
        } else {
            self.values.clear();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleModel {
    Stable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationInfoModel {
    pub known_pack_info: Option<String>,
    pub lifecycle: LifecycleModel,
}

impl RegistrationInfoModel {
    pub const BUILT_IN: Self = Self {
        known_pack_info: None,
        lifecycle: LifecycleModel::Stable,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderModel {
    id: usize,
    value_id: usize,
    keys: BTreeSet<String>,
    tags: BTreeSet<String>,
}

impl HolderModel {
    pub fn new(
        id: usize,
        value_id: usize,
        keys: impl IntoIterator<Item = impl Into<String>>,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            id,
            value_id,
            keys: keys.into_iter().map(Into::into).collect(),
            tags: tags.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSetModel {
    holder_ids: BTreeSet<usize>,
}

impl HolderSetModel {
    pub fn new(holder_ids: impl IntoIterator<Item = usize>) -> Self {
        Self {
            holder_ids: holder_ids.into_iter().collect(),
        }
    }

    pub fn contains(&self, holder: &HolderModel) -> bool {
        self.holder_ids.contains(&holder.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedInstanceModel {
    holder: HolderModel,
}

impl TypedInstanceModel {
    pub fn new(holder: HolderModel) -> Self {
        Self { holder }
    }

    pub fn tags(&self) -> BTreeSet<String> {
        self.holder.tags.clone()
    }

    pub fn is_tag(&self, tag: &str) -> bool {
        self.holder.tags.contains(tag)
    }

    pub fn is_set(&self, set: &HolderSetModel) -> bool {
        set.contains(&self.holder)
    }

    pub fn is_raw_type(&self, value_id: usize) -> bool {
        self.holder.value_id == value_id
    }

    pub fn is_holder(&self, holder_id: usize) -> bool {
        self.holder.id == holder_id
    }

    pub fn is_key(&self, key: &str) -> bool {
        self.holder.keys.contains(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UuidModel([u8; 16]);

impl UuidModel {
    pub fn from_int_array(words: [i32; 4]) -> Self {
        let mut bytes = [0u8; 16];
        for (index, word) in words.into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        Self(bytes)
    }

    pub fn to_int_array(self) -> [i32; 4] {
        let mut words = [0i32; 4];
        for (index, word) in words.iter_mut().enumerate() {
            let offset = index * 4;
            *word = i32::from_be_bytes([
                self.0[offset],
                self.0[offset + 1],
                self.0[offset + 2],
                self.0[offset + 3],
            ]);
        }
        words
    }

    pub fn to_byte_array(self) -> [u8; 16] {
        self.0
    }

    pub fn parse_strict(value: &str) -> Result<Self, String> {
        if value.len() != 36 {
            return Err(format!("Invalid UUID {value}: invalid length"));
        }
        for index in [8usize, 13, 18, 23] {
            if value.as_bytes()[index] != b'-' {
                return Err(format!("Invalid UUID {value}: expected hyphen at {index}"));
            }
        }
        let compact: String = value.chars().filter(|ch| *ch != '-').collect();
        Self::parse_hex_32(&compact).map_err(|error| format!("Invalid UUID {value}: {error}"))
    }

    pub fn parse_lenient_authlib(value: &str) -> Result<Self, String> {
        let compact: String = value.chars().filter(|ch| *ch != '-').collect();
        Self::parse_hex_32(&compact)
    }

    pub fn read_uuid(words: &[i32]) -> Result<Self, String> {
        let words: [i32; 4] = words.try_into().map_err(|_| {
            format!(
                "Could not read UUID. Expected int-array of length 4, got {}.",
                words.len()
            )
        })?;
        Ok(Self::from_int_array(words))
    }

    pub fn create_offline_player_uuid(name: &str) -> Self {
        let mut bytes = md5(format!("OfflinePlayer:{name}").as_bytes());
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Self(bytes)
    }

    fn parse_hex_32(value: &str) -> Result<Self, String> {
        if value.len() != 32 {
            return Err("invalid length".to_string());
        }
        let mut bytes = [0u8; 16];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let high = hex_value(value.as_bytes()[index * 2])?;
            let low = hex_value(value.as_bytes()[index * 2 + 1])?;
            *byte = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for UuidModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                write!(formatter, "-")?;
            }
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3iModel {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3iModel {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset_codec_validate(self, max_offset_per_axis: i32) -> Result<Self, String> {
        if self.x.abs() < max_offset_per_axis
            && self.y.abs() < max_offset_per_axis
            && self.z.abs() < max_offset_per_axis
        {
            Ok(self)
        } else {
            Err(format!(
                "Position out of range, expected at most {}: {}",
                max_offset_per_axis,
                self.to_short_string()
            ))
        }
    }

    pub fn hash_code(self) -> i32 {
        self.y
            .wrapping_add(self.z.wrapping_mul(31))
            .wrapping_mul(31)
            .wrapping_add(self.x)
    }

    pub fn compare_to(self, other: Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Equal => match self.z.cmp(&other.z) {
                Ordering::Equal => self.x.cmp(&other.x),
                z_order => z_order,
            },
            y_order => y_order,
        }
    }

    pub fn offset(self, x: i32, y: i32, z: i32) -> Self {
        if x == 0 && y == 0 && z == 0 {
            self
        } else {
            Self::new(self.x + x, self.y + y, self.z + z)
        }
    }

    pub fn subtract(self, other: Self) -> Self {
        self.offset(-other.x, -other.y, -other.z)
    }

    pub fn multiply(self, scale: i32) -> Self {
        match scale {
            0 => Self::ZERO,
            1 => self,
            _ => Self::new(self.x * scale, self.y * scale, self.z * scale),
        }
    }

    pub fn multiply_axes(self, x_scale: i32, y_scale: i32, z_scale: i32) -> Self {
        Self::new(self.x * x_scale, self.y * y_scale, self.z * z_scale)
    }

    pub fn relative(self, direction: CoreDirectionModel, steps: i32) -> Self {
        if steps == 0 {
            return self;
        }
        let (x, y, z) = direction.step();
        Self::new(self.x + x * steps, self.y + y * steps, self.z + z * steps)
    }

    pub fn relative_axis(self, axis: CoreAxisModel, steps: i32) -> Self {
        if steps == 0 {
            return self;
        }
        let x = axis.choose_i32(steps, 0, 0);
        let y = axis.choose_i32(0, steps, 0);
        let z = axis.choose_i32(0, 0, steps);
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn closer_than(self, other: Self, distance: f64) -> bool {
        self.dist_sqr(other) < distance * distance
    }

    pub fn closer_to_center_than(self, other: impl PositionModel, distance: f64) -> bool {
        self.dist_to_center_sqr(other) < distance * distance
    }

    pub fn dist_sqr(self, other: Self) -> f64 {
        self.dist_to_low_corner_sqr(other.x as f64, other.y as f64, other.z as f64)
    }

    pub fn dist_to_center_sqr(self, other: impl PositionModel) -> f64 {
        self.dist_to_center_sqr_xyz(other.x(), other.y(), other.z())
    }

    pub fn dist_to_center_sqr_xyz(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x as f64 + 0.5 - x;
        let dy = self.y as f64 + 0.5 - y;
        let dz = self.z as f64 + 0.5 - z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn dist_to_low_corner_sqr(self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x as f64 - x;
        let dy = self.y as f64 - y;
        let dz = self.z as f64 - z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn dist_manhattan(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    pub fn dist_chessboard(self, other: Self) -> i32 {
        (self.x - other.x)
            .abs()
            .max((self.y - other.y).abs())
            .max((self.z - other.z).abs())
    }

    pub fn get(self, axis: CoreAxisModel) -> i32 {
        axis.choose_i32(self.x, self.y, self.z)
    }

    pub fn to_mutable(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn to_short_string(self) -> String {
        format!("{}, {}, {}", self.x, self.y, self.z)
    }
}

fn hex_value(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(format!("invalid hex digit {}", value as char)),
    }
}

fn md5(input: &[u8]) -> [u8; 16] {
    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    for chunk in message.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            let start = i * 4;
            *word = u32::from_le_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | ((!b) & d), i)
            } else if i < 32 {
                ((d & b) | ((!d) & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | (!d)), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn client_asset_matches_texture_id_and_default_path_rules() {
        let downloaded = ClientTextureAssetModel::downloaded_texture(
            id("minecraft:entity/zombie"),
            "https://example.invalid/zombie.png",
        );
        assert_eq!(downloaded.id().to_string(), "minecraft:entity/zombie");
        assert_eq!(
            downloaded.texture_path().to_string(),
            "minecraft:entity/zombie"
        );

        let resource =
            ClientTextureAssetModel::resource_texture_from_id(id("minecraft:block/stone")).unwrap();
        assert_eq!(resource.id().to_string(), "minecraft:block/stone");
        assert_eq!(
            resource.texture_path().to_string(),
            "minecraft:textures/block/stone.png"
        );
        assert_eq!(ClientTextureAssetModel::default_field_name(), "asset_id");
        assert_eq!(resource.stream_value(), "minecraft:block/stone");

        let explicit = ClientTextureAssetModel::resource_texture(
            id("minecraft:stone_asset"),
            id("minecraft:textures/custom/stone.png"),
        );
        assert_eq!(explicit.id().to_string(), "minecraft:stone_asset");
        assert_eq!(
            explicit.texture_path().to_string(),
            "minecraft:textures/custom/stone.png"
        );
    }

    #[test]
    fn cloner_uses_source_then_target_context_and_factory_replaces_entries() {
        let cloner = ClonerModel::new("biome");
        assert_eq!(
            cloner
                .clone_value("plains", "registry_a", "registry_b")
                .unwrap(),
            "registry_b:plains"
        );
        assert_eq!(
            ClonerModel::failing_encode("biome")
                .clone_value("plains", "from", "to")
                .unwrap_err(),
            "Failed to encode: biome encode refused"
        );
        assert_eq!(
            ClonerModel::failing_decode("biome")
                .clone_value("plains", "from", "to")
                .unwrap_err(),
            "Failed to decode: biome decode refused"
        );

        let mut factory = ClonerFactoryModel::default();
        factory.add_codec("worldgen/biome", ClonerModel::new("first"));
        factory.add_codec("worldgen/biome", ClonerModel::new("replacement"));
        assert_eq!(
            factory
                .cloner("worldgen/biome")
                .unwrap()
                .clone_value("desert", "old", "new")
                .unwrap(),
            "new:desert"
        );
        assert!(factory.cloner("missing").is_none());
    }

    #[test]
    fn cursor3d_iterates_inclusive_box_in_java_order_and_classifies_boundaries() {
        let mut cursor = Cursor3DModel::new(10, 20, 30, 11, 21, 31);
        let mut visited = Vec::new();
        while cursor.advance() {
            visited.push((
                cursor.next_x(),
                cursor.next_y(),
                cursor.next_z(),
                cursor.next_type(),
            ));
        }
        assert_eq!(visited.len(), 8);
        assert_eq!(visited[0], (10, 20, 30, Cursor3DTypeModel::Corner));
        assert_eq!(visited[1], (11, 20, 30, Cursor3DTypeModel::Corner));
        assert_eq!(visited[2], (10, 21, 30, Cursor3DTypeModel::Corner));
        assert_eq!(visited[4], (10, 20, 31, Cursor3DTypeModel::Corner));
        assert_eq!(visited[7], (11, 21, 31, Cursor3DTypeModel::Corner));
        assert!(!cursor.advance());

        let mut wide = Cursor3DModel::new(0, 0, 0, 2, 2, 2);
        let mut types = BTreeMap::new();
        while wide.advance() {
            types.insert(
                (wide.next_x(), wide.next_y(), wide.next_z()),
                wide.next_type(),
            );
        }
        assert_eq!(types[&(1, 1, 1)], Cursor3DTypeModel::Inside);
        assert_eq!(types[&(0, 1, 1)], Cursor3DTypeModel::Face);
        assert_eq!(types[&(0, 0, 1)], Cursor3DTypeModel::Edge);
        assert_eq!(types[&(0, 0, 0)], Cursor3DTypeModel::Corner);
    }

    #[test]
    fn non_null_list_rejects_null_and_clear_preserves_default_sized_lists() {
        let mut empty = NonNullListModel::<String>::create_with_capacity(3);
        assert_eq!(empty.len(), 0);
        empty.add(0, Some("stone".to_string())).unwrap();
        assert_eq!(empty.set(0, Some("dirt".to_string())).unwrap(), "stone");
        assert_eq!(empty.remove(0), "dirt");
        assert_eq!(empty.add(0, None).unwrap_err(), "element");
        assert_eq!(empty.set(0, None).unwrap_err(), "element");
        empty.clear();
        assert_eq!(empty.len(), 0);

        let mut filled = NonNullListModel::with_size(3, Some("air".to_string())).unwrap();
        filled.set(1, Some("stone".to_string())).unwrap();
        filled.clear();
        assert_eq!(filled.len(), 3);
        assert_eq!(filled.get(0), "air");
        assert_eq!(filled.get(1), "air");
        assert_eq!(filled.get(2), "air");
        assert_eq!(
            NonNullListModel::<String>::with_size(1, None).unwrap_err(),
            "defaultValue"
        );

        let of = NonNullListModel::of(None, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(of.len(), 2);
        assert_eq!(NonNullListModel::<String>::create().len(), 0);
    }

    #[test]
    fn registration_info_and_typed_instance_delegate_to_holder_identity() {
        assert_eq!(RegistrationInfoModel::BUILT_IN.known_pack_info, None);
        assert_eq!(
            RegistrationInfoModel::BUILT_IN.lifecycle,
            LifecycleModel::Stable
        );
        assert!(core_package_null_marked());

        let instance = TypedInstanceModel::new(HolderModel::new(
            11,
            42,
            ["minecraft:stone"],
            ["mineable/pickaxe", "needs_stone_tool"],
        ));
        assert!(instance.is_tag("mineable/pickaxe"));
        assert_eq!(
            instance.tags(),
            BTreeSet::from([
                "mineable/pickaxe".to_string(),
                "needs_stone_tool".to_string()
            ])
        );
        assert!(instance.is_set(&HolderSetModel::new([11, 99])));
        assert!(!instance.is_set(&HolderSetModel::new([12])));
        assert!(instance.is_raw_type(42));
        assert!(!instance.is_raw_type(43));
        assert!(instance.is_holder(11));
        assert!(!instance.is_holder(12));
        assert!(instance.is_key("minecraft:stone"));
        assert!(!instance.is_key("minecraft:dirt"));
    }

    #[test]
    fn uuid_util_matches_int_array_string_authlib_and_offline_profile_rules() {
        let uuid = UuidModel::from_int_array([0x12345678, -0x65432110, 0x0fedcba9, -0x76543211]);
        assert_eq!(uuid.to_string(), "12345678-9abc-def0-0fed-cba989abcdef");
        assert_eq!(
            uuid.to_int_array(),
            [0x12345678, -0x65432110, 0x0fedcba9, -0x76543211]
        );
        assert_eq!(
            uuid.to_byte_array(),
            [
                0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x89, 0xab,
                0xcd, 0xef
            ]
        );
        assert_eq!(UuidModel::read_uuid(&uuid.to_int_array()).unwrap(), uuid);
        assert_eq!(
            UuidModel::read_uuid(&[1, 2, 3]).unwrap_err(),
            "Could not read UUID. Expected int-array of length 4, got 3."
        );
        assert_eq!(UuidModel::parse_strict(&uuid.to_string()).unwrap(), uuid);
        assert!(UuidModel::parse_strict("123456789abcdef00fedcba989abcdef").is_err());
        assert_eq!(
            UuidModel::parse_lenient_authlib("123456789abcdef00fedcba989abcdef").unwrap(),
            uuid
        );
        assert_eq!(
            UuidModel::create_offline_player_uuid("Steve").to_string(),
            "5627dd98-e6be-3c21-b8a8-e92344183641"
        );
        assert_eq!(
            UuidModel::create_offline_player_uuid("Alex").to_string(),
            "36532b5e-c442-3dbb-a24c-c7e55d0f979a"
        );
    }

    #[test]
    fn vec3i_matches_java_ordering_hash_offsets_and_distances() {
        let pos = Vec3iModel::new(1, 2, 3);
        assert_eq!(Vec3iModel::ZERO, Vec3iModel::new(0, 0, 0));
        assert_eq!(pos.hash_code(), 2946);
        assert_eq!(pos.compare_to(Vec3iModel::new(9, 2, 3)), Ordering::Less);
        assert_eq!(pos.compare_to(Vec3iModel::new(1, 1, 99)), Ordering::Greater);
        assert_eq!(pos.offset(0, 0, 0), pos);
        assert_eq!(pos.offset(4, -1, 2), Vec3iModel::new(5, 1, 5));
        assert_eq!(
            pos.subtract(Vec3iModel::new(2, 4, 6)),
            Vec3iModel::new(-1, -2, -3)
        );
        assert_eq!(pos.multiply(0), Vec3iModel::ZERO);
        assert_eq!(pos.multiply(1), pos);
        assert_eq!(pos.multiply(3), Vec3iModel::new(3, 6, 9));
        assert_eq!(pos.multiply_axes(2, 3, 4), Vec3iModel::new(2, 6, 12));
        assert_eq!(
            pos.relative(CoreDirectionModel::North, 2),
            Vec3iModel::new(1, 2, 1)
        );
        assert_eq!(pos.relative(CoreDirectionModel::Up, 0), pos);
        assert_eq!(
            pos.relative_axis(CoreAxisModel::Y, -3),
            Vec3iModel::new(1, -1, 3)
        );
        assert_eq!(
            pos.cross(Vec3iModel::new(4, 5, 6)),
            Vec3iModel::new(-3, 6, -3)
        );
        assert_eq!(pos.get(CoreAxisModel::Z), 3);
        assert_eq!(pos.to_mutable(), (1, 2, 3));
        assert_eq!(pos.to_short_string(), "1, 2, 3");
    }

    #[test]
    fn vec3i_matches_java_codec_bounds_and_distance_helpers() {
        let pos = Vec3iModel::new(1, 2, 3);
        assert_eq!(pos.offset_codec_validate(4).unwrap(), pos);
        assert_eq!(
            pos.offset_codec_validate(3).unwrap_err(),
            "Position out of range, expected at most 3: 1, 2, 3"
        );
        assert_eq!(pos.dist_sqr(Vec3iModel::new(4, 6, 3)), 25.0);
        assert_eq!(
            pos.dist_to_center_sqr(Vec3PositionModel {
                x: 1.5,
                y: 3.5,
                z: 6.5,
            }),
            10.0
        );
        assert_eq!(pos.dist_to_low_corner_sqr(2.0, 4.0, 6.0), 14.0);
        assert!(pos.closer_than(Vec3iModel::new(3, 2, 3), 2.1));
        assert!(!pos.closer_than(Vec3iModel::new(3, 2, 3), 2.0));
        assert!(pos.closer_to_center_than(
            Vec3PositionModel {
                x: 1.5,
                y: 2.5,
                z: 4.4,
            },
            2.0
        ));
        assert_eq!(pos.dist_manhattan(Vec3iModel::new(-2, 4, 9)), 11);
        assert_eq!(pos.dist_chessboard(Vec3iModel::new(-2, 4, 9)), 6);
    }
}
