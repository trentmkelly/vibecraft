use super::*;

pub fn palette_bits_for_size(palette_len: usize) -> usize {
    let needed = usize::BITS as usize - (palette_len.saturating_sub(1)).leading_zeros() as usize;
    needed.max(4)
}

pub fn pack_palette_indices(indices: &[u64], bits_per_entry: usize) -> Vec<i64> {
    let values_per_long = 64 / bits_per_entry;
    let mut packed = vec![0_u64; indices.len().div_ceil(values_per_long)];
    for (i, &value) in indices.iter().enumerate() {
        let word = i / values_per_long;
        let bit = (i % values_per_long) * bits_per_entry;
        packed[word] |= value << bit;
    }
    packed.into_iter().map(|w| w as i64).collect()
}

pub fn unpack_palette_indices(data: &[i64], bits_per_entry: usize, count: usize) -> Vec<u64> {
    let values_per_long = 64 / bits_per_entry;
    let mask = (1_u64 << bits_per_entry) - 1;
    let mut indices = vec![0_u64; count];
    for i in 0..count {
        let word = i / values_per_long;
        let bit = (i % values_per_long) * bits_per_entry;
        if word < data.len() {
            indices[i] = (data[word] as u64 >> bit) & mask;
        }
    }
    indices
}

impl PalettedContainer {
    pub fn single(entry: Tag, expected_entries: usize) -> Self {
        Self {
            palette: vec![entry],
            data: None,
            expected_entries,
        }
    }

    pub fn to_nbt(&self) -> Tag {
        let mut fields = vec![("palette".to_string(), Tag::List(self.palette.clone()))];
        if let Some(data) = &self.data {
            fields.push(("data".to_string(), Tag::LongArray(data.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn from_nbt(tag: &Tag, expected_entries: usize) -> Result<Self, String> {
        let compound = compound(tag)?;
        let palette = list_field(compound, "palette")?.to_vec();
        if palette.is_empty() {
            return Err("paletted container palette cannot be empty".to_string());
        }
        let data = match compound.iter().find(|(field_name, _)| field_name == "data") {
            Some((_name, Tag::LongArray(values))) => Some(values.clone()),
            Some((_name, _)) => {
                return Err("paletted container data must be a long array".to_string())
            }
            None => None,
        };
        Ok(Self {
            palette,
            data,
            expected_entries,
        })
    }

    pub fn get_entry(&self, index: usize) -> Option<&Tag> {
        if self.palette.len() == 1 {
            return self.palette.first();
        }
        let bits = palette_bits_for_size(self.palette.len());
        if index >= self.expected_entries {
            return None;
        }
        let values_per_long = 64 / bits;
        let word = index / values_per_long;
        let bit = (index % values_per_long) * bits;
        let mask = (1_u64 << bits) - 1;
        let palette_idx =
            ((self.data.as_deref()?.get(word).copied()? as u64 >> bit) & mask) as usize;
        self.palette.get(palette_idx)
    }

    pub fn set_entry(&mut self, index: usize, entry: Tag) {
        self.set_entry_ref(index, &entry);
    }

    pub fn ensure_palette_entry(&mut self, entry: &Tag) -> usize {
        let old_palette_len = self.palette.len();
        let palette_idx = match self.palette.iter().position(|e| e == entry) {
            Some(i) => i,
            None => {
                self.palette.push(entry.clone());
                self.palette.len() - 1
            }
        };
        let old_bits = palette_bits_for_size(old_palette_len.max(1));
        let bits = palette_bits_for_size(self.palette.len());
        if old_bits != bits {
            let count = self.expected_entries;
            let indices = self
                .data
                .as_ref()
                .map(|d| unpack_palette_indices(d, old_bits, count))
                .unwrap_or_else(|| vec![0_u64; count]);
            self.data = Some(pack_palette_indices(&indices, bits));
        }
        palette_idx
    }

    pub fn set_palette_index(&mut self, index: usize, palette_idx: usize) {
        if index >= self.expected_entries || palette_idx >= self.palette.len() {
            return;
        }
        if self.palette.len() == 1 {
            self.data = None;
            return;
        }
        let bits = palette_bits_for_size(self.palette.len());
        let values_per_long = 64 / bits;
        let word_len = self.expected_entries.div_ceil(values_per_long);
        let data = self.data.get_or_insert_with(|| vec![0_i64; word_len]);
        if data.len() < word_len {
            data.resize(word_len, 0);
        }
        let word = index / values_per_long;
        let bit = (index % values_per_long) * bits;
        let mask = ((1_u64 << bits) - 1) << bit;
        let current = data[word] as u64;
        data[word] = ((current & !mask) | ((palette_idx as u64) << bit)) as i64;
    }

    pub fn set_entry_ref(&mut self, index: usize, entry: &Tag) {
        if index >= self.expected_entries {
            return;
        }
        let old_palette_len = self.palette.len();
        let palette_idx = match self.palette.iter().position(|e| e == entry) {
            Some(i) => i,
            None => {
                self.palette.push(entry.clone());
                self.palette.len() - 1
            }
        };
        if self.palette.len() == 1 {
            self.data = None;
            return;
        }
        let bits = palette_bits_for_size(self.palette.len());
        let count = self.expected_entries;
        let old_bits = palette_bits_for_size(old_palette_len.max(1));
        if old_bits == bits {
            let values_per_long = 64 / bits;
            let word_len = count.div_ceil(values_per_long);
            let data = self.data.get_or_insert_with(|| vec![0_i64; word_len]);
            if data.len() < word_len {
                data.resize(word_len, 0);
            }
            let word = index / values_per_long;
            let bit = (index % values_per_long) * bits;
            let mask = ((1_u64 << bits) - 1) << bit;
            let current = data[word] as u64;
            data[word] = ((current & !mask) | ((palette_idx as u64) << bit)) as i64;
            return;
        }

        let mut indices = self
            .data
            .as_ref()
            .map(|d| unpack_palette_indices(d, old_bits, count))
            .unwrap_or_else(|| vec![0_u64; count]);
        indices[index] = palette_idx as u64;
        self.data = Some(pack_palette_indices(&indices, bits));
    }
}

pub(super) fn paletted_container_entry_from_nbt(
    tag: &Tag,
    expected_entries: usize,
    index: usize,
) -> Option<&Tag> {
    if index >= expected_entries {
        return None;
    }
    let Tag::Compound(fields) = tag else {
        return None;
    };
    let palette = fields.iter().find_map(|(name, value)| {
        (name == "palette").then_some(value).and_then(|value| {
            if let Tag::List(entries) = value {
                Some(entries)
            } else {
                None
            }
        })
    })?;
    if palette.len() == 1 {
        return palette.first();
    }
    let data = fields.iter().find_map(|(name, value)| {
        (name == "data").then_some(value).and_then(|value| {
            if let Tag::LongArray(data) = value {
                Some(data)
            } else {
                None
            }
        })
    })?;
    let bits = palette_bits_for_size(palette.len());
    let values_per_long = 64 / bits;
    let word = index / values_per_long;
    let bit = (index % values_per_long) * bits;
    let mask = (1_u64 << bits) - 1;
    let palette_idx = ((data.get(word).copied()? as u64 >> bit) & mask) as usize;
    palette.get(palette_idx)
}

pub(super) fn set_paletted_container_entry_in_nbt(
    tag: &mut Tag,
    expected_entries: usize,
    index: usize,
    entry: &Tag,
) -> bool {
    if index >= expected_entries {
        return false;
    }
    let Tag::Compound(fields) = tag else {
        return false;
    };
    let Some(palette_field_index) = fields.iter().position(|(name, _)| name == "palette") else {
        return false;
    };
    let Tag::List(palette) = &mut fields[palette_field_index].1 else {
        return false;
    };
    if palette.is_empty() {
        return false;
    }

    let old_palette_len = palette.len();
    let palette_idx = match palette.iter().position(|candidate| candidate == entry) {
        Some(index) => index,
        None => {
            palette.push(entry.clone());
            palette.len() - 1
        }
    };
    if palette.len() == 1 {
        fields.retain(|(name, _)| name != "data");
        return true;
    }

    let old_bits = palette_bits_for_size(old_palette_len.max(1));
    let bits = palette_bits_for_size(palette.len());
    let values_per_long = 64 / bits;
    let word_len = expected_entries.div_ceil(values_per_long);
    let data_field_index = fields.iter().position(|(name, _)| name == "data");

    if old_bits == bits {
        let data = match data_field_index {
            Some(index) => match &mut fields[index].1 {
                Tag::LongArray(data) => data,
                _ => return false,
            },
            None => {
                fields.push(("data".to_string(), Tag::LongArray(vec![0_i64; word_len])));
                match &mut fields.last_mut().expect("data field was just inserted").1 {
                    Tag::LongArray(data) => data,
                    _ => unreachable!(),
                }
            }
        };
        if data.len() < word_len {
            data.resize(word_len, 0);
        }
        let word = index / values_per_long;
        let bit = (index % values_per_long) * bits;
        let mask = ((1_u64 << bits) - 1) << bit;
        let current = data[word] as u64;
        data[word] = ((current & !mask) | ((palette_idx as u64) << bit)) as i64;
        return true;
    }

    let old_data = data_field_index.and_then(|index| match &fields[index].1 {
        Tag::LongArray(data) => Some(data.as_slice()),
        _ => None,
    });
    let mut indices = old_data
        .map(|data| unpack_palette_indices(data, old_bits, expected_entries))
        .unwrap_or_else(|| vec![0_u64; expected_entries]);
    indices[index] = palette_idx as u64;
    let packed = pack_palette_indices(&indices, bits);
    if let Some(index) = data_field_index {
        fields[index].1 = Tag::LongArray(packed);
    } else {
        fields.push(("data".to_string(), Tag::LongArray(packed)));
    }
    true
}

