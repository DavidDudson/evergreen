---
name: wiki-enrich
description: Enrich an existing wiki character page with transcript-sourced content (history, quotes, relationships, appearances). Use when the user asks to flesh out, enrich, or add detail to a character's wiki page from session transcripts.
tools: Read, Write, Edit, Bash, Glob, Grep, Agent, WebFetch
---

# Wiki Enrich Skill

Appends new sections to an **existing** character page on the EvergreenNova
fandom wiki, sourced from raw session transcripts. Never replaces existing
content — only adds new sections under a `== Bot Conjecture ==` heading.

---

## Inputs

The user provides:
- **Character name** (required) — e.g. `Queen Maeve`, `Mordred`, `Nimue`
- **Session range** (optional) — e.g. `11-30`. Defaults to all available sessions.

---

## Workflow

### Step 1 — Fetch the existing wiki page

```bash
curl -s "https://evergeennova.fandom.com/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&titles=PAGE_TITLE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.revisions[0].slots.main["*"]'
```

Save this content. You need it to:
1. Confirm the page exists (abort if missing — this skill is append-only)
2. Know what content is **already there** so you don't duplicate it
3. Identify whether a `== Bot Conjecture ==` section already exists

### Step 2 — Identify relevant sessions

Search the raw transcripts for mentions of the character:

```bash
grep -li "CHARACTER_NAME" research/sessions/yt-scripts/Session*
```

Also check the session summaries:

```bash
grep -li "CHARACTER_NAME" research/sessions/summaries/Session*.md
```

This gives you the session numbers where the character appears. Read those
summaries first (fast), then dig into the raw transcripts for quotes and
timestamps.

### Step 3 — Extract the YouTube video ID for each session

Each summary file contains a YouTube link on line 4:
```
**YouTube:** [Session N](https://www.youtube.com/watch?v=VIDEO_ID)
```

Each raw transcript file contains timestamp links like:
```
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]
```

Use these to create timestamped links in the wiki content.

### Step 4 — Analyse transcripts for new content

For each relevant session, extract:

#### History
Events, actions, revelations, and decisions involving the character. Focus on
**what they did** and **what happened to them**. Write in past tense, narrative
style, matching the tone of existing wiki pages (see Queen Maeve example).

#### Quotes
Direct speech attributed to the character. Auto-captions are noisy — only
include quotes you're confident about. Format:

```
{{Quote|Quote text here.|Source — [[Session N]] [video_url (timestamp)]}}
```

If the `{{Quote}}` template doesn't exist, use a simple blockquote:
```
:"''Quote text here.''" — [[Session N]]
```

#### Relationships
New relationship information revealed in the session. Who do they interact
with? What is the nature of the relationship? Has it changed?

Format as sub-sections under `=== Character Name ===`.

#### Birth / Death
Any information about the character's birth, origin, death, resurrection,
or transformation. Include session references.

#### Appearances in the Campaign
A chronological list of sessions where the character appeared or was
significantly referenced. Format like the Queen Maeve example:

```
=== [[Session N]] ===
Brief narrative of what happened with this character in this session.
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (timestamp)] for key moments.
```

### Step 5 — Diff against existing content

Before generating output, compare your findings against the existing wiki
page content from Step 1. **Remove anything that is already covered.**

Rules:
- If a session appearance is already documented → skip it
- If a relationship is already described → skip it (unless new info changes it)
- If a quote is already on the page → skip it
- If history is already covered → skip it
- Only include genuinely **new** information

### Step 6 — Format the wikitext

All new content goes under `== Bot Conjecture ==` at the bottom of the page,
before `==References==` and category tags.

Structure:

```mediawiki
== Bot Conjecture ==
{{Bot conjecture|date=YYYY-MM-DD}}

=== History (Bot) ===
Narrative history content here. Reference sessions with <ref>[[Session N]]</ref>.

=== Quotes (Bot) ===
:"''Quote text here.''" — [[Session N]] [https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]

:"''Another quote.''" — [[Session N]]

=== Relationships (Bot) ===
==== [[Character Name]] ====
Description of the relationship. <ref>[[Session N]]</ref>

=== Birth / Death (Bot) ===
Information about origin, birth, death, resurrection if applicable.

=== Appearances in the Campaign (Bot) ===
==== [[Session N]] ====
What happened with this character in this session.
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (timestamp)]
```

**Only include sections that have new content.** If there are no new quotes,
omit the Quotes section entirely. Don't add empty sections.

If a `== Bot Conjecture ==` section already exists on the page, append new
`===` subsections to it — never create a second `== Bot Conjecture ==`.

### Step 7 — Present for approval

Show the user an approval block:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WIKI ENRICH APPROVAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page     : <wiki page title>
Action   : APPEND TO "Bot Conjecture"
Sections : <list of sections being added>
Sessions : <session numbers referenced>

--- PREVIEW ---
<the wikitext that will be appended>
--- END PREVIEW ---

Type YES to push, NO to skip, or EDIT to modify.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Wait for explicit confirmation before pushing.

### Step 8 — Push

Use `wiki_push.sh` or the MediaWiki API to append content:

```bash
# Option A: If wiki_push.sh supports the page
./scripts/wiki_push.sh push "<Page Title>" "<local file>"

# Option B: Direct API append (for section-level edits)
CSRF=$(curl -s -c cookies -b cookies \
  "${API}?action=query&meta=tokens&type=csrf&format=json" \
  | jq -r '.query.tokens.csrftoken')

curl -s -c cookies -b cookies -X POST \
  --data-urlencode "title=<Page Title>" \
  --data-urlencode "appendtext=<NEW WIKITEXT>" \
  --data-urlencode "summary=Bot enrichment: added history/quotes/relationships from session transcripts" \
  --data-urlencode "token=${CSRF}" \
  "${API}?action=edit&format=json"
```

The `appendtext` parameter adds content to the END of the page without
touching existing content.

---

## Wiki Link Formatting

When referencing other characters, locations, or sessions, use wiki links:

- Characters: `[[Character Name]]` or `[[Wiki Page Title|Display Name]]`
- Sessions: `[[Session N]]`
- Locations: `[[Location Name]]`
- Items: `[[Item Name]]`

For YouTube timestamps, use external links:
```
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]
```

---

## Matching Character Names in Transcripts

Auto-captions garble names. Use these known mappings:

| Caption Name | Actual Character |
|---|---|
| Della, Drizella | Drizella Tremaine |
| Morid, Mid, Mordred | Mordred |
| Silla, Sibylla | Sibylla |
| Big B, Bigby | Bigby |
| Nimway | Nimue |
| Savan | Savvan |
| Memphis | Memphis Cadwallader |
| Morana, Morgana | Morgana Le Fay |
| Carlos | The Beast King |
| Dean Maricosa | Dean Marikosa |

When searching for a character, try both the real name and known garbled
variants.

---

## Rules

1. **Never replace existing content.** This skill only appends.
2. **All content goes under `== Bot Conjecture ==`.** Use `{{Bot conjecture|date=YYYY-MM-DD}}`.
3. **Skip content that already exists on the page.** Diff first.
4. **Mark uncertain quotes with (?)** — auto-captions are noisy.
5. **Include session references** for every claim using `<ref>[[Session N]]</ref>`.
6. **Include YouTube timestamps** where possible for key moments and quotes.
7. **Write in the wiki's existing tone** — past tense, narrative, encyclopaedic.
8. **Approval required** before any API call (inherited from wiki-update skill).
9. **One character at a time.** If the user asks for multiple, process sequentially.

---

## Quick Reference

```bash
# Check if a page exists
curl -s "https://evergeennova.fandom.com/api.php?action=query&titles=PAGE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.missing // "exists"'

# Fetch existing page content
curl -s "https://evergeennova.fandom.com/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&titles=PAGE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.revisions[0].slots.main["*"]'

# Find character mentions in transcripts
grep -li "CHARACTER" research/sessions/yt-scripts/Session*

# Find character mentions in summaries
grep -li "CHARACTER" research/sessions/summaries/Session*.md

# Get YouTube ID from a summary
grep "YouTube:" research/sessions/summaries/SessionN.md

# Get timestamps from raw transcript
grep "youtube.com" research/sessions/yt-scripts/SessionN | head -5
```

---

## Example Output

For a character like Nimue, the appended content might look like:

```mediawiki
== Bot Conjecture ==
{{Bot conjecture|date=2026-03-22}}

=== History (Bot) ===
During [[Session 11]], Nimue conducted a divination ritual at [[Ersatz University]]'s Observatory during the [[Parallax]], attempting to determine when the [[Sunderance]] would occur. She had mind-controlled [[Savvan]] to operate the machine and imprisoned [[Dean Marikosa]] in a back chamber. She cast an 8th-level dominate spell (DC 40) on [[Mordred]], turning him against the party.<ref>[[Session 11]]</ref>

When [[Drizella]] nearly died during the fight, Nimue's true self — a peaceful, drowning figure trapped within her corrupted outer shell — communicated via Message spell: ''"The Waters of Pendragon can free me."''<ref>[[Session 11]]</ref>

=== Quotes (Bot) ===
:"''Leave this place. I have work that must be done. I am commanded by a greater power.''" — [[Session 11]] [https://www.youtube.com/watch?v=u9AJ43VXt2Y&t=250 (00:04:10)]

:"''You might have known the glory I seek if you would serve the order.''" — [[Session 11]]

=== Relationships (Bot) ===
==== [[Savvan]] ====
Nimue mind-controlled Savvan to operate her divination machine during the Parallax ritual. He fought the control, urging the party: "Destroy the machine!"<ref>[[Session 11]]</ref>

=== Appearances in the Campaign (Bot) ===
==== [[Session 11]] ====
Nimue attacked the party at the Ersatz University Observatory, conducting a Parallax divination ritual. She mind-controlled Mordred and stole Drizella's mask before escaping. Caliburn's scabbard was recovered from the wreckage. [https://www.youtube.com/watch?v=u9AJ43VXt2Y&t=62 (00:01:02)]
```
