//! Shared value behind Java LevelData.RespawnData and the default-spawn packet.
//!
//! The factory normalizes player-supplied rotations. The persisted codec instead
//! validates its input, and the network record retains raw float values. Keeping
//! those boundaries separate matches the Java factory, MAP_CODEC, and STREAM_CODEC.
use crate::registry::Identifier;
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelRespawnData {
    pub dimension: Identifier,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for LevelRespawnData {
    fn default() -> Self {
        Self {
            dimension: Identifier::with_default_namespace("overworld")
                .unwrap_or_else(|error| panic!("invalid built-in dimension: {error}")),
            x: 0,
            y: 0,
            z: 0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl LevelRespawnData {
    /// Java RespawnData.of: copy the position, wrap yaw, and clamp pitch.
    pub fn of(dimension: Identifier, position: (i32, i32, i32), yaw: f32, pitch: f32) -> Self {
        let mut yaw = yaw % 360.0;
        if yaw >= 180.0 {
            yaw -= 360.0;
        }
        if yaw < -180.0 {
            yaw += 360.0;
        }
        Self {
            dimension,
            x: position.0,
            y: position.1,
            z: position.2,
            yaw,
            pitch: pitch.clamp(-90.0, 90.0),
        }
    }

    /// GlobalPos.MAP_CODEC fields and the two required, bounded float fields.
    pub fn to_nbt(&self) -> Result<Tag, String> {
        validate_rotation("yaw", self.yaw, 180.0)?;
        validate_rotation("pitch", self.pitch, 90.0)?;
        Ok(Tag::Compound(vec![
            (
                "dimension".to_owned(),
                Tag::String(self.dimension.to_string()),
            ),
            (
                "pos".to_owned(),
                Tag::IntArray(vec![self.x, self.y, self.z]),
            ),
            ("yaw".to_owned(), Tag::Float(self.yaw)),
            ("pitch".to_owned(), Tag::Float(self.pitch)),
        ]))
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let Tag::Compound(fields) = tag else {
            return Err("respawn data must be a compound".to_owned());
        };
        let field = |name: &str| {
            fields
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value)
                .ok_or_else(|| format!("missing respawn field {name}"))
        };
        let Tag::String(dimension) = field("dimension")? else {
            return Err("respawn dimension must be a string".to_owned());
        };
        let position = position_values(field("pos")?)?;
        let yaw = float_value(field("yaw")?)?;
        let pitch = float_value(field("pitch")?)?;
        validate_rotation("yaw", yaw, 180.0)?;
        validate_rotation("pitch", pitch, 90.0)?;
        Ok(Self {
            dimension: Identifier::parse(dimension)?,
            x: position[0],
            y: position[1],
            z: position[2],
            yaw,
            pitch,
        })
    }
}

fn validate_rotation(name: &str, value: f32, limit: f32) -> Result<(), String> {
    if (-limit..=limit).contains(&value) {
        Ok(())
    } else {
        Err(format!("respawn {name} must be in [{}, {limit}]", -limit))
    }
}

// NbtOps reads Number values, so FLOAT and INT_STREAM accept all numeric tags.
fn float_value(tag: &Tag) -> Result<f32, String> {
    tag.numeric_value()
        .map(|number| number.float_value())
        .ok_or_else(|| "respawn rotation must be numeric".to_owned())
}

fn position_values(tag: &Tag) -> Result<[i32; 3], String> {
    use crate::storage::nbt::nbt_ops::{NbtOpsModel, NbtOpsResult};
    let values = match NbtOpsModel::INSTANCE.get_int_stream(tag) {
        NbtOpsResult::Success(values) => values,
        NbtOpsResult::Error { message, .. } => return Err(message),
    };
    values
        .try_into()
        .map_err(|_| "respawn position must contain exactly three coordinates".to_owned())
}

#[cfg(test)]
mod tests;
