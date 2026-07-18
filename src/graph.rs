//! Depth-first graph traversal matching Minecraft's cycle-aware utility.
#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Returns true on a cycle; otherwise appends each completed node in reverse topological order.
pub fn depth_first_search<T: Eq + Hash + Clone>(edges: &HashMap<T, HashSet<T>>, discovered: &mut HashSet<T>, currently_visiting: &mut HashSet<T>, reverse_topological_order: &mut impl FnMut(T), current: T) -> bool {
    if discovered.contains(&current) { return false; }
    if currently_visiting.contains(&current) { return true; }
    currently_visiting.insert(current.clone());
    if let Some(next) = edges.get(&current) { for next in next { if depth_first_search(edges, discovered, currently_visiting, reverse_topological_order, next.clone()) { return true; } } }
    currently_visiting.remove(&current); discovered.insert(current.clone()); reverse_topological_order(current); false
}
#[cfg(test)] mod tests { use super::*; #[cfg(vibecraft_has_decompiled_sources)] #[test] fn source(){const JAVA:&str=vibecraft_java_source!("/net/minecraft/util/Graph.java");assert_eq!(JAVA.lines().count(),36);for s in ["discovered.contains(current)","currentlyVisiting.contains(current)","currentlyVisiting.remove(current)","reverseTopologicalOrder.accept(current)"]{assert!(JAVA.contains(s));}} #[test] fn orders_acyclic_and_reports_cycles(){let mut e=HashMap::new();e.insert("a",HashSet::from(["b"]));e.insert("b",HashSet::from(["c"]));let(mut d,mut v,mut order)=(HashSet::new(),HashSet::new(),Vec::new());assert!(!depth_first_search(&e,&mut d,&mut v,&mut |n|order.push(n),"a"));assert_eq!(order,["c","b","a"]);e.insert("c",HashSet::from(["a"]));let(mut d,mut v)=(HashSet::new(),HashSet::new());assert!(depth_first_search(&e,&mut d,&mut v,&mut |_|{},"a"));}}
