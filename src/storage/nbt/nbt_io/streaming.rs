//! Direct NBT visitor traversal. Skipped containers never become Tag trees.
use super::*;
use crate::storage::nbt::{read_i32, read_u16};

pub(super) fn parse<R: Read, V: NbtStreamTagVisitor>(
    reader: &mut R,
    visitor: &mut V,
) -> io::Result<()> {
    let id = read_u8(reader)?;
    match visitor.visit_root_entry(tag_type(i32::from(id))) {
        StreamValueResult::Halt => Ok(()),
        StreamValueResult::Break => {
            if id != 0 {
                skip_string(reader)?;
            }
            skip(reader, id, 0)
        }
        StreamValueResult::Continue => {
            if id != 0 {
                skip_string(reader)?;
            }
            payload(reader, id, visitor, 0).map(|_| ())
        }
    }
}

fn container_depth(depth: usize) -> io::Result<()> {
    if depth >= DEFAULT_MAX_NBT_DEPTH {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "NBT depth limit exceeded",
        ))
    } else {
        Ok(())
    }
}

fn payload<R: Read, V: NbtStreamTagVisitor>(
    reader: &mut R,
    id: u8,
    visitor: &mut V,
    depth: usize,
) -> io::Result<StreamValueResult> {
    match id {
        9 => {
            container_depth(depth)?;
            list(reader, visitor, depth + 1)
        }
        10 => {
            container_depth(depth)?;
            compound(reader, visitor, depth + 1)
        }
        _ => visit_tag_payload(
            &Tag::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH)?,
            visitor,
        ),
    }
}

fn compound<R: Read, V: NbtStreamTagVisitor>(
    reader: &mut R,
    visitor: &mut V,
    depth: usize,
) -> io::Result<StreamValueResult> {
    loop {
        let id = read_u8(reader)?;
        if id == 0 {
            return Ok(visitor.visit_container_end());
        }
        let ty = tag_type(i32::from(id));
        let action = match visitor.visit_entry(ty.clone()) {
            StreamEntryResult::Enter => visitor.visit_named_entry(ty, &read_string(reader)?),
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            action => {
                skip_string(reader)?;
                action
            }
        };
        match action {
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            StreamEntryResult::Skip => skip(reader, id, depth)?,
            StreamEntryResult::Break => {
                skip(reader, id, depth)?;
                skip_compound_contents(reader, depth)?;
                return Ok(visitor.visit_container_end());
            }
            StreamEntryResult::Enter => {
                // CompoundTag.parseCompound propagates HALT; value BREAK is
                // handled by the child parser and does not skip sibling entries.
                if payload(reader, id, visitor, depth)? == StreamValueResult::Halt {
                    return Ok(StreamValueResult::Halt);
                }
            }
        }
    }
}

fn list<R: Read, V: NbtStreamTagVisitor>(
    reader: &mut R,
    visitor: &mut V,
    depth: usize,
) -> io::Result<StreamValueResult> {
    let id = read_u8(reader)?;
    let count = read_i32(reader)?;
    if count < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ListTag length cannot be negative: {count}"),
        ));
    }
    match visitor.visit_list(tag_type(i32::from(id)), count as usize) {
        StreamValueResult::Halt => return Ok(StreamValueResult::Halt),
        StreamValueResult::Break => {
            skip_many(reader, id, count, depth)?;
            return Ok(visitor.visit_container_end());
        }
        StreamValueResult::Continue => {}
    }
    for index in 0..count {
        let stop = match visitor.visit_element(tag_type(i32::from(id)), index as usize) {
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            StreamEntryResult::Skip => {
                skip(reader, id, depth)?;
                false
            }
            StreamEntryResult::Break => {
                skip(reader, id, depth)?;
                true
            }
            StreamEntryResult::Enter => match payload(reader, id, visitor, depth)? {
                StreamValueResult::Halt => return Ok(StreamValueResult::Halt),
                StreamValueResult::Break => true,
                StreamValueResult::Continue => false,
            },
        };
        if stop {
            skip_many(reader, id, count - index - 1, depth)?;
            break;
        }
    }
    Ok(visitor.visit_container_end())
}

// DataInput.skipBytes skips up to n bytes and treats negative amounts as zero.
// Fixed-size TagType bulk skips also use Java's wrapping int multiplication.
fn discard<R: Read>(reader: &mut R, bytes: i32) -> io::Result<()> {
    io::copy(&mut reader.take(bytes.max(0) as u64), &mut io::sink()).map(|_| ())
}

fn skip_string<R: Read>(reader: &mut R) -> io::Result<()> {
    let len = read_u16(reader)?;
    discard(reader, i32::from(len))
}

fn fixed_size(id: u8) -> Option<i32> {
    match id {
        0 => Some(0),
        1 => Some(1),
        2 => Some(2),
        3 | 5 => Some(4),
        4 | 6 => Some(8),
        _ => None,
    }
}

fn skip_many<R: Read>(reader: &mut R, id: u8, count: i32, depth: usize) -> io::Result<()> {
    if id > 12 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid tag id: {id}"),
        ));
    }
    if let Some(size) = fixed_size(id) {
        return discard(reader, size.wrapping_mul(count));
    }
    for _ in 0..count {
        skip(reader, id, depth)?;
    }
    Ok(())
}

fn skip<R: Read>(reader: &mut R, id: u8, depth: usize) -> io::Result<()> {
    if let Some(size) = fixed_size(id) {
        return discard(reader, size);
    }
    match id {
        7 | 11 | 12 => {
            let count = read_i32(reader)?;
            let size = match id {
                7 => 1,
                11 => 4,
                _ => 8,
            };
            discard(reader, count.wrapping_mul(size))
        }
        8 => skip_string(reader),
        9 => {
            container_depth(depth)?;
            let element = read_u8(reader)?;
            let count = read_i32(reader)?;
            skip_many(reader, element, count, depth + 1)
        }
        10 => {
            container_depth(depth)?;
            skip_compound_contents(reader, depth + 1)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid tag id: {id}"),
        )),
    }
}

fn skip_compound_contents<R: Read>(reader: &mut R, depth: usize) -> io::Result<()> {
    loop {
        let id = read_u8(reader)?;
        if id == 0 {
            return Ok(());
        }
        skip_string(reader)?;
        skip(reader, id, depth)?;
    }
}

#[cfg(test)]
mod tests;
