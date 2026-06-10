# Vibecraft

Given the full decompiled source of the Minecraft 26.1.2 server, can LLM coding agents recreate the entire thing at 1:1 parity in Rust? The main purpose of this repo is to answer that question. So far, it appears the answer is probably going to be yes, although it remains to be seen how *well* they can do it.

The primary contributing model has been GPT-5.5 with medium reasoning effort. Claude has done a bit of work as well. The main workhorse models may change in the future as new models come out.

Safety note: I haven't manually inspected *any* of this code, and I'm not going to, either. The only human-written or human-read file in this entire repo is README.md. Run the code at your own risk.

## Current status

As of June 10th, the game is at least somewhat playable. Worldgen parity is almost 1:1 with Java but not quite. Crafting works, but not all blocks can be placed. Haven't checked the nether or the end at all.

## Building from source

Ask your favorite LLM to do it for you lmao
