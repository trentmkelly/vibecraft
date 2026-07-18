//! ID-to-value lookup builders matching Minecraft's `ByIdMap`.
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutOfBoundsStrategy { Zero, Wrap, Clamp }

pub fn sparse<T: Clone>(id: impl Fn(&T) -> i32, values: &[T], default: T) -> Result<impl Fn(i32) -> T, String> {
    let pairs = checked_pairs(id, values)?;
    Ok(move |key| pairs.iter().find(|(entry, _)| *entry == key).map_or_else(|| default.clone(), |(_, value)| value.clone()))
}

pub fn continuous<T: Clone>(id: impl Fn(&T) -> i32, values: &[T], strategy: OutOfBoundsStrategy) -> Result<impl Fn(i32) -> T, String> {
    if values.is_empty() { return Err("Empty value list".to_string()); }
    let mut sorted: Vec<Option<T>> = vec![None; values.len()];
    for value in values { let key=id(value); if key < 0 || key as usize >= values.len() { return Err(format!("Values are not continous, found index {key} for value")); } let slot=&mut sorted[key as usize]; if slot.is_some() { return Err(format!("Duplicate entry on id {key}")); } *slot=Some(value.clone()); }
    let sorted: Vec<T> = sorted.into_iter().enumerate().map(|(index, value)| value.ok_or_else(|| format!("Missing value at index: {index}"))).collect::<Result<_,_>>()?;
    let length=sorted.len() as i32;
    Ok(move |key| { let index=match strategy { OutOfBoundsStrategy::Zero => if key < 0 || key >= length { 0 } else { key }, OutOfBoundsStrategy::Wrap => key.rem_euclid(length), OutOfBoundsStrategy::Clamp => key.clamp(0,length-1) }; sorted[index as usize].clone() })
}

fn checked_pairs<T: Clone>(id: impl Fn(&T)->i32, values:&[T])->Result<Vec<(i32,T)>,String>{ if values.is_empty(){return Err("Empty value list".into())} let mut result=Vec::new(); for value in values {let key=id(value);if result.iter().any(|(seen,_)|*seen==key){return Err(format!("Duplicate entry on id {key}"))}result.push((key,value.clone()));}Ok(result)}

#[cfg(test)] mod tests { use super::*; #[cfg(vibecraft_has_decompiled_sources)] #[test] fn source(){const JAVA:&str=vibecraft_java_source!("/net/minecraft/util/ByIdMap.java");assert_eq!(JAVA.lines().count(),85);for fragment in ["Empty value list","Duplicate entry on id ","Values are not continous","Missing value at index:","case WRAP","case CLAMP"]{assert!(JAVA.contains(fragment));}} #[test] fn lookup_and_strategies(){let sparse=sparse(|v:&(i32,&str)|v.0,&[(2,"two")],(0,"zero")).unwrap_or_else(|e|panic!("{e}"));assert_eq!(sparse(2).1,"two");assert_eq!(sparse(8).1,"zero");for (strategy,expected) in [(OutOfBoundsStrategy::Zero,0),(OutOfBoundsStrategy::Wrap,2),(OutOfBoundsStrategy::Clamp,2)] {let map=continuous(|v:&i32|*v,&[0,1,2],strategy).unwrap_or_else(|e|panic!("{e}"));assert_eq!(map(5),expected);}} }
