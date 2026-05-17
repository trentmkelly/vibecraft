use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRecord {
    pub name: String,
    pub entity_type: String,
    pub uuid: String,
    pub player: bool,
    pub level: String,
    pub team: Option<String>,
    pub tags: Vec<String>,
    pub scores: BTreeMap<String, i32>,
    pub nbt: BTreeMap<String, String>,
    pub predicates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityWithPosition {
    pub entity: EntityRecord,
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub base: SelectorBase,
    pub includes_entities: bool,
    pub current_entity: bool,
    pub world_limited: bool,
    pub limit: usize,
    pub sort: SelectorSort,
    pub name: Option<Inverted<String>>,
    pub entity_type: Option<Inverted<String>>,
    pub team: Option<Inverted<String>>,
    pub tag: Option<Inverted<String>>,
    pub distance: Option<RangeF64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub dz: Option<f64>,
    pub scores: BTreeMap<String, RangeI32>,
    pub nbt: BTreeMap<String, String>,
    pub predicate: Option<Inverted<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorBase {
    NearestPlayer,
    AllPlayers,
    AllEntities,
    CurrentEntity,
    RandomPlayer,
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorSort {
    Nearest,
    Furthest,
    Random,
    Arbitrary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inverted<T> {
    pub value: T,
    pub inverted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeF64 {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeI32 {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorError {
    Empty,
    InvalidBase,
    InvalidOption(String),
    InvalidRange(String),
    InvalidLimit,
}

impl Selector {
    pub fn parse(input: &str) -> Result<Self, SelectorError> {
        if input.is_empty() {
            return Err(SelectorError::Empty);
        }
        if !input.starts_with('@') {
            return Ok(name_selector(input));
        }
        let (head, options) = if let Some(start) = input.find('[') {
            if !input.ends_with(']') {
                return Err(SelectorError::InvalidBase);
            }
            (&input[..start], Some(&input[start + 1..input.len() - 1]))
        } else {
            (input, None)
        };
        let mut selector = match head {
            "@p" => base_selector(SelectorBase::NearestPlayer, false, 1, SelectorSort::Nearest),
            "@a" => base_selector(
                SelectorBase::AllPlayers,
                false,
                usize::MAX,
                SelectorSort::Arbitrary,
            ),
            "@e" => base_selector(
                SelectorBase::AllEntities,
                true,
                usize::MAX,
                SelectorSort::Arbitrary,
            ),
            "@s" => {
                let mut selector = base_selector(
                    SelectorBase::CurrentEntity,
                    true,
                    1,
                    SelectorSort::Arbitrary,
                );
                selector.current_entity = true;
                selector
            }
            "@r" => base_selector(SelectorBase::RandomPlayer, false, 1, SelectorSort::Random),
            _ => return Err(SelectorError::InvalidBase),
        };
        if let Some(options) = options {
            for option in split_options(options) {
                apply_option(&mut selector, option)?;
            }
        }
        Ok(selector)
    }

    pub fn select(
        &self,
        entities: &[EntityWithPosition],
        source_position: Vec3,
        source_level: &str,
        current_entity: Option<&str>,
    ) -> Vec<EntityRecord> {
        let origin = Vec3 {
            x: self.x.unwrap_or(source_position.x),
            y: self.y.unwrap_or(source_position.y),
            z: self.z.unwrap_or(source_position.z),
        };
        let mut selected: Vec<_> = entities
            .iter()
            .filter(|entry| self.matches(entry, origin, source_level, current_entity))
            .cloned()
            .collect();
        match self.sort {
            SelectorSort::Nearest => selected.sort_by(|a, b| {
                distance_sqr(a.position, origin).total_cmp(&distance_sqr(b.position, origin))
            }),
            SelectorSort::Furthest => selected.sort_by(|a, b| {
                distance_sqr(b.position, origin).total_cmp(&distance_sqr(a.position, origin))
            }),
            SelectorSort::Random => selected.sort_by(|a, b| {
                stable_random_key(&a.entity.uuid).cmp(&stable_random_key(&b.entity.uuid))
            }),
            SelectorSort::Arbitrary => {}
        }
        selected
            .into_iter()
            .take(self.limit)
            .map(|entry| entry.entity)
            .collect()
    }

    fn matches(
        &self,
        entry: &EntityWithPosition,
        origin: Vec3,
        source_level: &str,
        current_entity: Option<&str>,
    ) -> bool {
        let entity = &entry.entity;
        if !self.includes_entities && !entity.player {
            return false;
        }
        if self.world_limited && entity.level != source_level {
            return false;
        }
        if self.current_entity && Some(entity.uuid.as_str()) != current_entity {
            return false;
        }
        if let SelectorBase::Name(name) = &self.base {
            return entity.name == *name || entity.uuid == *name;
        }
        if !matches_inverted(&self.name, &entity.name) {
            return false;
        }
        if !matches_inverted(&self.entity_type, &entity.entity_type) {
            return false;
        }
        if let Some(team) = &self.team {
            let value = entity.team.as_deref().unwrap_or("");
            if (value == team.value) == team.inverted {
                return false;
            }
        }
        if let Some(tag) = &self.tag {
            let contains = entity.tags.iter().any(|entry| entry == &tag.value);
            if contains == tag.inverted {
                return false;
            }
        }
        if let Some(range) = self.distance {
            let distance = distance_sqr(entry.position, origin).sqrt();
            if !range.contains(distance) {
                return false;
            }
        }
        if !self.in_box(entry.position, origin) {
            return false;
        }
        if self
            .scores
            .iter()
            .any(|(objective, range)| !range.contains(*entity.scores.get(objective).unwrap_or(&0)))
        {
            return false;
        }
        if self
            .nbt
            .iter()
            .any(|(key, value)| entity.nbt.get(key) != Some(value))
        {
            return false;
        }
        if let Some(predicate) = &self.predicate {
            let contains = entity
                .predicates
                .iter()
                .any(|entry| entry == &predicate.value);
            if contains == predicate.inverted {
                return false;
            }
        }
        true
    }

    fn in_box(&self, position: Vec3, origin: Vec3) -> bool {
        if self.dx.is_none() && self.dy.is_none() && self.dz.is_none() {
            return true;
        }
        let dx = self.dx.unwrap_or(0.0);
        let dy = self.dy.unwrap_or(0.0);
        let dz = self.dz.unwrap_or(0.0);
        between(position.x, origin.x, origin.x + dx)
            && between(position.y, origin.y, origin.y + dy)
            && between(position.z, origin.z, origin.z + dz)
    }
}

impl RangeF64 {
    fn contains(self, value: f64) -> bool {
        self.min.map(|min| value >= min).unwrap_or(true)
            && self.max.map(|max| value <= max).unwrap_or(true)
    }
}

impl RangeI32 {
    fn contains(self, value: i32) -> bool {
        self.min.map(|min| value >= min).unwrap_or(true)
            && self.max.map(|max| value <= max).unwrap_or(true)
    }
}

fn base_selector(
    base: SelectorBase,
    includes_entities: bool,
    limit: usize,
    sort: SelectorSort,
) -> Selector {
    Selector {
        base,
        includes_entities,
        current_entity: false,
        world_limited: false,
        limit,
        sort,
        name: None,
        entity_type: None,
        team: None,
        tag: None,
        distance: None,
        x: None,
        y: None,
        z: None,
        dx: None,
        dy: None,
        dz: None,
        scores: BTreeMap::new(),
        nbt: BTreeMap::new(),
        predicate: None,
    }
}

fn name_selector(input: &str) -> Selector {
    let mut selector = base_selector(
        SelectorBase::Name(input.to_string()),
        true,
        1,
        SelectorSort::Arbitrary,
    );
    selector.world_limited = false;
    selector
}

fn apply_option(selector: &mut Selector, option: &str) -> Result<(), SelectorError> {
    let Some((key, value)) = option.split_once('=') else {
        return Err(SelectorError::InvalidOption(option.to_string()));
    };
    match key {
        "name" => selector.name = Some(parse_inverted_string(value)),
        "type" => selector.entity_type = Some(parse_inverted_string(value)),
        "team" => selector.team = Some(parse_inverted_string(value)),
        "tag" => selector.tag = Some(parse_inverted_string(value)),
        "predicate" => selector.predicate = Some(parse_inverted_string(value)),
        "distance" => selector.distance = Some(parse_range_f64(value)?),
        "x" => selector.x = Some(parse_f64(value)?),
        "y" => selector.y = Some(parse_f64(value)?),
        "z" => selector.z = Some(parse_f64(value)?),
        "dx" => {
            selector.dx = Some(parse_f64(value)?);
            selector.world_limited = true;
        }
        "dy" => {
            selector.dy = Some(parse_f64(value)?);
            selector.world_limited = true;
        }
        "dz" => {
            selector.dz = Some(parse_f64(value)?);
            selector.world_limited = true;
        }
        "limit" => {
            let limit = value
                .parse::<usize>()
                .map_err(|_| SelectorError::InvalidLimit)?;
            if limit == 0 {
                return Err(SelectorError::InvalidLimit);
            }
            selector.limit = limit;
        }
        "sort" => {
            selector.sort = match value {
                "nearest" => SelectorSort::Nearest,
                "furthest" => SelectorSort::Furthest,
                "random" => SelectorSort::Random,
                "arbitrary" => SelectorSort::Arbitrary,
                _ => return Err(SelectorError::InvalidOption(option.to_string())),
            }
        }
        "scores" => {
            for (name, range) in parse_map(value)? {
                selector.scores.insert(name, parse_range_i32(&range)?);
            }
        }
        "nbt" => {
            for (name, expected) in parse_map(value)? {
                selector.nbt.insert(name, expected);
            }
        }
        _ => return Err(SelectorError::InvalidOption(key.to_string())),
    }
    Ok(())
}

fn split_options(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (idx, ch) in input.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(input[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start <= input.len() {
        let tail = input[start..].trim();
        if !tail.is_empty() {
            parts.push(tail);
        }
    }
    parts
}

fn parse_map(input: &str) -> Result<Vec<(String, String)>, SelectorError> {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(SelectorError::InvalidOption(input.to_string()));
    }
    let body = &trimmed[1..trimmed.len() - 1];
    if body.is_empty() {
        return Ok(Vec::new());
    }
    body.split(',')
        .map(|entry| {
            let (key, value) = entry
                .split_once('=')
                .or_else(|| entry.split_once(':'))
                .ok_or_else(|| SelectorError::InvalidOption(entry.to_string()))?;
            Ok((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn parse_inverted_string(value: &str) -> Inverted<String> {
    if let Some(rest) = value.strip_prefix('!') {
        Inverted {
            value: rest.to_string(),
            inverted: true,
        }
    } else {
        Inverted {
            value: value.to_string(),
            inverted: false,
        }
    }
}

fn parse_range_f64(value: &str) -> Result<RangeF64, SelectorError> {
    if let Some((min, max)) = value.split_once("..") {
        Ok(RangeF64 {
            min: if min.is_empty() {
                None
            } else {
                Some(parse_f64(min)?)
            },
            max: if max.is_empty() {
                None
            } else {
                Some(parse_f64(max)?)
            },
        })
    } else {
        let exact = parse_f64(value)?;
        Ok(RangeF64 {
            min: Some(exact),
            max: Some(exact),
        })
    }
}

fn parse_range_i32(value: &str) -> Result<RangeI32, SelectorError> {
    if let Some((min, max)) = value.split_once("..") {
        Ok(RangeI32 {
            min: if min.is_empty() {
                None
            } else {
                Some(parse_i32(min)?)
            },
            max: if max.is_empty() {
                None
            } else {
                Some(parse_i32(max)?)
            },
        })
    } else {
        let exact = parse_i32(value)?;
        Ok(RangeI32 {
            min: Some(exact),
            max: Some(exact),
        })
    }
}

fn parse_f64(value: &str) -> Result<f64, SelectorError> {
    value
        .parse::<f64>()
        .map_err(|_| SelectorError::InvalidRange(value.to_string()))
}

fn parse_i32(value: &str) -> Result<i32, SelectorError> {
    value
        .parse::<i32>()
        .map_err(|_| SelectorError::InvalidRange(value.to_string()))
}

fn matches_inverted(expected: &Option<Inverted<String>>, actual: &str) -> bool {
    match expected {
        Some(expected) => (actual == expected.value) != expected.inverted,
        None => true,
    }
}

fn between(value: f64, first: f64, second: f64) -> bool {
    value >= first.min(second) && value <= first.max(second)
}

fn distance_sqr(a: Vec3, b: Vec3) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}

fn stable_random_key(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(
        name: &str,
        uuid: &str,
        player: bool,
        entity_type: &str,
        position: Vec3,
    ) -> EntityWithPosition {
        EntityWithPosition {
            entity: EntityRecord {
                name: name.to_string(),
                entity_type: entity_type.to_string(),
                uuid: uuid.to_string(),
                player,
                level: "overworld".to_string(),
                team: None,
                tags: Vec::new(),
                scores: BTreeMap::new(),
                nbt: BTreeMap::new(),
                predicates: Vec::new(),
            },
            position,
        }
    }

    #[test]
    fn parses_selector_bases_limits_sorts_and_ranges() {
        let selector =
            Selector::parse("@e[type=minecraft:zombie,distance=..10,limit=2,sort=nearest]")
                .unwrap();
        assert_eq!(selector.base, SelectorBase::AllEntities);
        assert!(selector.includes_entities);
        assert_eq!(selector.limit, 2);
        assert_eq!(selector.sort, SelectorSort::Nearest);
        assert_eq!(selector.distance.unwrap().max, Some(10.0));
        assert_eq!(
            selector.entity_type,
            Some(Inverted {
                value: "minecraft:zombie".to_string(),
                inverted: false,
            })
        );
        assert_eq!(
            Selector::parse("@a[limit=0]"),
            Err(SelectorError::InvalidLimit)
        );
    }

    #[test]
    fn filters_players_entities_names_types_teams_tags_and_current_entity() {
        let mut alex = entity(
            "Alex",
            "u1",
            true,
            "minecraft:player",
            Vec3 {
                x: 1.0,
                y: 64.0,
                z: 0.0,
            },
        );
        alex.entity.team = Some("red".to_string());
        alex.entity.tags.push("builder".to_string());
        let zombie = entity(
            "Zombie",
            "u2",
            false,
            "minecraft:zombie",
            Vec3 {
                x: 4.0,
                y: 64.0,
                z: 0.0,
            },
        );
        let entities = vec![alex.clone(), zombie.clone()];

        assert_eq!(
            Selector::parse("Alex").unwrap().select(
                &entities,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                "overworld",
                None
            ),
            vec![alex.entity.clone()]
        );
        assert_eq!(
            Selector::parse("@a[team=red,tag=builder]").unwrap().select(
                &entities,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                "overworld",
                None
            ),
            vec![alex.entity.clone()]
        );
        assert_eq!(
            Selector::parse("@e[type=minecraft:zombie]")
                .unwrap()
                .select(
                    &entities,
                    Vec3 {
                        x: 0.0,
                        y: 64.0,
                        z: 0.0
                    },
                    "overworld",
                    None
                ),
            vec![zombie.entity.clone()]
        );
        assert_eq!(
            Selector::parse("@s").unwrap().select(
                &entities,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                "overworld",
                Some("u1")
            ),
            vec![alex.entity.clone()]
        );
    }

    #[test]
    fn score_nbt_predicate_and_box_filters_match_expected_entities() {
        let mut alex = entity(
            "Alex",
            "u1",
            true,
            "minecraft:player",
            Vec3 {
                x: 2.0,
                y: 65.0,
                z: 2.0,
            },
        );
        alex.entity.scores.insert("kills".to_string(), 5);
        alex.entity
            .nbt
            .insert("Health".to_string(), "20".to_string());
        alex.entity.predicates.push("minecraft:can_fly".to_string());
        let steve = entity(
            "Steve",
            "u2",
            true,
            "minecraft:player",
            Vec3 {
                x: 10.0,
                y: 65.0,
                z: 10.0,
            },
        );
        let entities = vec![alex.clone(), steve];

        let selector = Selector::parse("@a[x=0,y=64,z=0,dx=3,dy=2,dz=3,scores={kills=3..},nbt={Health=20},predicate=minecraft:can_fly]").unwrap();
        assert_eq!(
            selector.select(
                &entities,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                "overworld",
                None
            ),
            vec![alex.entity]
        );
    }

    #[test]
    fn sorting_and_limits_choose_nearest_furthest_and_stable_random_order() {
        let entities = vec![
            entity(
                "Near",
                "u1",
                true,
                "minecraft:player",
                Vec3 {
                    x: 1.0,
                    y: 64.0,
                    z: 0.0,
                },
            ),
            entity(
                "Far",
                "u2",
                true,
                "minecraft:player",
                Vec3 {
                    x: 9.0,
                    y: 64.0,
                    z: 0.0,
                },
            ),
            entity(
                "Mid",
                "u3",
                true,
                "minecraft:player",
                Vec3 {
                    x: 5.0,
                    y: 64.0,
                    z: 0.0,
                },
            ),
        ];
        let origin = Vec3 {
            x: 0.0,
            y: 64.0,
            z: 0.0,
        };
        assert_eq!(
            Selector::parse("@a[sort=nearest,limit=1]").unwrap().select(
                &entities,
                origin,
                "overworld",
                None
            )[0]
            .name,
            "Near"
        );
        assert_eq!(
            Selector::parse("@a[sort=furthest,limit=1]")
                .unwrap()
                .select(&entities, origin, "overworld", None)[0]
                .name,
            "Far"
        );
        let first = Selector::parse("@a[sort=random,limit=3]").unwrap().select(
            &entities,
            origin,
            "overworld",
            None,
        );
        let second = Selector::parse("@a[sort=random,limit=3]").unwrap().select(
            &entities,
            origin,
            "overworld",
            None,
        );
        assert_eq!(first, second);
    }
}
