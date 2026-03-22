---
name: upload-yt-script
description: Upload a session's YouTube transcript to the EvergreenNova fandom wiki, analyse it for missing wiki content, update research files, and create a session summary.
tools: Read, Write, Edit, Bash, Glob, Grep, Agent, WebFetch
---

# Upload YouTube Script to Wiki

This skill uploads a session's YouTube transcript to the EvergreenNova fandom wiki, analyses it against existing wiki content, updates research files with new findings, and creates a per-session summary.

---

## Inputs

The user provides:
- **Session number** (required) — e.g. `42`
- **YouTube video URL** (optional) — if not provided, extract it from the transcript file's timestamp links

---

## Workflow

### Step 1 — Locate and validate the transcript

The transcript file lives at:
```
research/sessions/yt-scripts/Session <N>
```

Files are plain text (no extension) containing auto-generated YouTube captions with inline timestamp links like:
```
[(00:01:02)](https://www.youtube.com/watch?v=VIDEO_ID&t=62)
```

Read the file. If it doesn't exist, tell the user and stop.

### Step 2 — Extract the YouTube video URL

If the user didn't provide a YouTube URL, extract the video ID from the first timestamp link in the transcript:

```
https://www.youtube.com/watch?v=VIDEO_ID
```

Strip the `&t=` parameter to get the clean video URL.

### Step 3 — Analyse the transcript against existing wiki and research

Read the existing research files to understand what's already documented:
- `research/sessions/session_summary.md` — existing session recaps
- `research/characters/npc_list.md` — known NPCs
- `research/characters/player_characters.md` — known PCs
- `research/world/factions.md` — known factions
- `research/world/factions_alignment.md` — faction alignment map
- `research/world/lore_events.md` — lore timeline
- `research/quests/quest_list.md` — known quests
- Any character-specific files in `research/characters/`

Then analyse the transcript for:

1. **New or missing characters** — NPCs, villains, allies, or named entities not in the wiki
2. **Important plot points** — major revelations, betrayals, deaths, alliances, discoveries
3. **Notable quotes** — memorable or plot-significant dialogue (attribute to speaker)
4. **Faction developments** — new factions mentioned, faction relationships changing, political events
5. **Quest hooks** — new quests started, quest progress, quest completions
6. **Lore reveals** — world history, magical systems, prophecies, artefact lore
7. **Location introductions** — new places visited or described

### Step 4 — Identify the "hero question" and answers

Look through the transcript for any segment where the DM or an NPC (often Galen) poses a moral/philosophical question to the players, typically with multiple-choice answers that grant alignment points. Document:
- The exact question (or best reconstruction from the auto-captions)
- The available answers and which faction alignment each grants
- Which answer(s) the players chose

If no hero question is found in this session, note that.

### Step 5 — Create a session summary

Create a summary file at:
```
research/sessions/summaries/Session <N>.md
```

Format:

```markdown
# Session <N> Summary

**Date:** <date if discernible from transcript or session_summary.md>
**YouTube:** [Session <N>](https://www.youtube.com/watch?v=VIDEO_ID)
**Arc:** <arc name from session_summary.md>

## Synopsis

<2-4 paragraph narrative summary of what happened in the session>

## Key Events

- <bulleted list of major plot beats>

## Characters Introduced/Appearing

- **<Name>** — <brief description and role in this session>

## Notable Quotes

- "<quote>" — <speaker>

## Hero Question

<The question, answers, and player choices — or "No hero question this session.">

## Faction Developments

- <any faction-related changes>

## Quest Progress

- <quest updates>

## Lore Reveals

- <new lore information>
```

### Step 6 — Update local research files

Update the following files **additively** with new information from the transcript:

| File | What to add |
|------|-------------|
| `research/characters/npc_list.md` | New NPCs with brief descriptions |
| `research/characters/player_characters.md` | New PC details or developments |
| `research/characters/<name>.md` | Create or update character-specific files for significant characters |
| `research/world/factions.md` | New factions or faction developments |
| `research/world/factions_alignment.md` | Alignment relationship changes |
| `research/world/lore_events.md` | New lore events with session reference |
| `research/quests/quest_list.md` | New quests or quest progress updates |

**Rules for research file updates:**
- Be additive only — never delete or overwrite existing content
- Clearly mark new additions with a comment like `<!-- Added from Session N -->`
- If unsure about a detail (auto-captions are noisy), mark it with `(?)` and a note about transcript quality
- Attribute all additions to the session number

### Step 7 — Create the transcript wiki page

Create a new file at:
```
research/sessions/yt-scripts/Session <N>.md
```

Format it as:

```markdown
{{Bot-created|date=YYYY-MM-DD|source=evergreen repo session transcript}}

= Session <N> — YouTube Transcript =

YouTube video: [https://www.youtube.com/watch?v=VIDEO_ID]

== Transcript ==

<transcript content, cleaned up:>
- Keep the timestamp links as-is (they're useful for navigation)
- Wrap in a single <pre> block if pandoc is not available
- Do NOT rewrite or summarise the transcript — upload it verbatim
```

### Step 8 — Add the page mapping to wiki_push.sh

Add an entry to the `PAGE_MAP` associative array in `scripts/wiki_push.sh`:

```bash
["${RESEARCH}/sessions/yt-scripts/Session <N>.md"]="Session <N>/Transcript"
```

The wiki page title follows the pattern `Session <N>/Transcript` (a subpage of the session page).

### Step 9 — Update session_summary.md

In `research/sessions/session_summary.md`, find the entry for Session `<N>`. Add the following if not already present:

1. **YouTube video link** — Add a line like:
   ```
   **YouTube:** [Session <N>](https://www.youtube.com/watch?v=VIDEO_ID)
   ```

2. **Transcript page link** — Add a line like:
   ```
   **Transcript:** [[Session <N>/Transcript]]
   ```

If the session is only referenced as part of an arc range (e.g. "Sessions 19–21") and doesn't have its own subsection, add a bullet point under the relevant arc section.

### Step 10 — Update the wiki

Use the `/wiki-update` skill rules for all wiki pushes. For each page to push:

**New pages** (transcript page, any new character pages):
- Apply Rule 2 — `{{Bot-created}}` banner
- Present approval block, wait for confirmation

**Existing pages** (session summary, character pages, quest list, etc.):
- Apply Rule 1 — append under `== Bot Conjecture ==` section
- Present approval block, wait for confirmation

Push order:
1. Transcript page (new)
2. Session Summary page (existing — append YouTube/transcript links)
3. Any character, faction, quest, or lore pages that need updates

Present each approval block one at a time.

### Step 11 — Report findings

After all updates, present a summary to the user:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SESSION <N> ANALYSIS COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
YouTube    : <url>
Hero Q     : <question summary or "none found">

New characters found    : <count>
Plot points documented  : <count>
Quotes captured         : <count>
Research files updated  : <list>
Wiki pages pushed       : <list>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Rules

- All wiki-update skill rules (Rule 1, 2, 3) apply — especially **mandatory manual approval** before any API call.
- **Never modify the raw transcript content.** Upload it verbatim. The `.md` file is the wiki-formatted version; the original extensionless file is the raw source.
- **Be additive only** when updating existing research files. Never delete existing content.
- **Mark uncertainty.** Auto-generated captions are noisy — flag uncertain names, quotes, or details with `(?)`.
- **Attribute everything** to the session number so findings can be traced.
- If multiple sessions are requested, process them one at a time.
- Use subagents to parallelise reading existing research files and analysing the transcript.

---

## Quick Reference

```bash
# Check if a transcript exists
ls "research/sessions/yt-scripts/Session 42"

# Extract video ID from first timestamp
grep -oP 'youtube\.com/watch\?v=\K[^&)]+' "research/sessions/yt-scripts/Session 42" | head -1

# Push after approval
just wiki-push-one "Session 42/Transcript" "research/sessions/yt-scripts/Session 42.md"
```
