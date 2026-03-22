---
name: wiki-enrich
description: Enrich an existing wiki character page with transcript-sourced content (history, quotes, relationships, appearances). Use when the user asks to flesh out, enrich, or add detail to a character's wiki page from session transcripts.
tools: Read, Write, Edit, Bash, Glob, Grep, Agent, WebFetch
---

# Wiki Enrich Skill

Appends new sections to an **existing** character page on the EvergreenNova
fandom wiki, sourced from raw session transcripts. Never replaces existing
content. New content goes under `== Bot Conjecture ==`.

---

## Inputs

The user provides:
- **Character name** (required)
- **Session range** (optional, defaults to all)

---

## Workflow

### Step 1 -- Fetch and save the existing wiki page

```bash
curl -s "https://evergeennova.fandom.com/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&titles=PAGE_TITLE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.revisions[0].slots.main["*"]' > /tmp/page_existing.txt
```

Save this to a file. You need the **complete original content** to:
1. Confirm the page exists (abort if missing)
2. Know what is already documented (to avoid duplicates)
3. Identify sessions already covered in Appearances
4. Detect existing `== Bot Conjecture ==` section

### Step 2 -- Find relevant sessions

```bash
# Summaries (fast, read these first)
grep -li "CHARACTER" research/sessions/summaries/Session*.md

# Raw transcripts (for quotes and timestamps)
grep -li "CHARACTER" research/sessions/yt-scripts/Session*
```

Try garbled name variants too (see name mappings below).

### Step 3 -- Get YouTube video IDs and timestamps

**From summaries** (line 4):
```
**YouTube:** [Session N](https://www.youtube.com/watch?v=VIDEO_ID)
```

**From raw transcripts** (no `.md` extension):
```
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]
```

**CRITICAL:** Always use the video ID from the raw transcript itself, not
from the summary. They may differ. Search the transcript for character-name
mentions near key events to find the right timestamp.

### Step 4 -- Extract new content

For each session NOT already on the page, extract:

**History:** What they did, what happened to them. Past tense, narrative.

**Quotes:** Direct speech from the character. Mark uncertain with (?).

**Relationships:** Who they interact with, how.

**Appearances:** Brief narrative of involvement per session.

**Every claim must have a YouTube timestamp where one can be found.** Search
the raw transcript for the character name near the relevant dialogue. Place
timestamps inline after the claim, before the `<ref>` tag.

### Step 5 -- Diff against existing content

Compare findings against the existing page. Remove duplicates:
- Session already in Appearances? Skip it.
- Relationship already described? Skip (unless new info).
- Quote already present? Skip.
- History already covered? Skip.

### Step 6 -- Build the complete page

**DO NOT use `appendtext`.** It appends after footer elements and breaks
the page structure.

Instead, build the complete page:

1. Read the original page content from Step 1
2. Strip footer elements: `{{Template:Nav-Characters}}`, `{{Template:Nav-Deities}}`,
   `[[Category:Characters]]`, `[[Category:Deities]]`, `== References ==`, `<references />`
3. Append the Bot Conjecture section
4. Re-add footer at the very end

**Bot Conjecture structure** (must match this exactly):

```mediawiki
== Bot Conjecture ==
{{Bot conjecture|date=YYYY-MM-DD}}

=== History (Bot) ===
Narrative content. [https://www.youtube.com/watch?v=ID&t=SEC (HH:MM:SS)] <ref>[[Session NN]]</ref>

=== Quotes (Bot) ===
:"''Quote text.''" [https://www.youtube.com/watch?v=ID&t=SEC (HH:MM:SS)] <ref>[[Session NN]]</ref>

=== Relationships (Bot) ===
==== [[Character Name]] ====
Description. [https://www.youtube.com/watch?v=ID&t=SEC (HH:MM:SS)] <ref>[[Session NN]]</ref>

=== Appearances in the Campaign (Bot) ===
==== [[Session NN]] ====
What happened. [https://www.youtube.com/watch?v=ID&t=SEC (HH:MM:SS)]
```

**Sub-section naming:** Always use `(Bot)` suffix: `=== History (Bot) ===`,
`=== Relationships (Bot) ===`, `=== Appearances in the Campaign (Bot) ===`.

**Appearance entries:** Use `==== [[Session NN]] ====` (four `=`).

**Only include sections with content.** Omit empty sections.

If `== Bot Conjecture ==` already exists, append new `===` sub-sections
to it. Never create a duplicate.

### Step 7 -- Verify before presenting

Before showing the approval block, verify:
1. **Original content preserved.** The new file must be LONGER than the
   original. If it's shorter, something was lost. Abort and investigate.
2. **No em/en dashes.** Search for unicode chars. Rephrase if found.
3. **Session zero-padding.** Sessions 1-9 must be `[[Session 01]]` to `[[Session 09]]`.
4. **YouTube timestamps present.** Every Appearances entry and every
   significant claim should have a timestamp if one was found in the transcript.
5. **Footer at end.** `{{Template:Nav-Characters}}`, `[[Category:Characters]]`,
   `== References ==`, `<references />` must be the last 4 lines.

### Step 8 -- Present for approval

```
Page     : <title>
Action   : APPEND TO "Bot Conjecture"
Sections : History, Relationships, Appearances
Sessions : S07, S22, S32 ...
Original : NN lines
New      : NN lines (+NN added)
Timestamps: NN YouTube links
```

### Step 9 -- Push

```bash
./scripts/wiki_push.sh push "<Page Title>" /tmp/page_complete.txt
```

The file must contain `{{` or `==` headings for wiki_push.sh to detect
it as raw wikitext and skip pandoc.

---

## Footer Element Reference

Different pages use different footer templates. Detect from the original:

```
{{Template:Nav-Characters}}   -- most character pages
{{Template:Nav-Deities}}      -- deity pages
[[Category:Characters]]       -- most pages
[[Category:Deities]]          -- deity pages
== References ==               -- or ==References==
<references />
```

Strip ALL of these from the original, append Bot Conjecture, then re-add
the same footer elements at the end.

---

## Timestamp Searching Tips

1. Search the raw transcript (no `.md` extension) not the wiki copy
2. Use `grep -i "character_name" research/sessions/yt-scripts/SessionNN`
3. Timestamps appear inline: `[https://www.youtube.com/watch?v=ID&t=SEC (HH:MM:SS)]`
4. Search for keywords near the character name to find the right moment
5. If the character name is garbled in captions, search for related terms
   (location, event, other character in the scene)
6. Place timestamps after the relevant claim, before `<ref>`:
   `...the ritual. [https://...&t=7763 (02:09:23)] <ref>[[Session 08]]</ref>`

---

## Character Name Mappings (Auto-Caption Garbling)

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
| Queen Ma, Queen Mave | Queen Maeve |
| Silver Lady | Queen Maeve |
| Goth, Mother Goth | Dame Gothel |

---

## Rules

1. **Never replace existing content.** Build the complete page: original + Bot Conjecture + footer.
2. **Verify line count.** New file must be longer than original. If shorter, content was lost.
3. **All content goes under `== Bot Conjecture ==`.** Use `{{Bot conjecture|date=YYYY-MM-DD}}`.
4. **Sub-sections use `(Bot)` suffix.** `=== History (Bot) ===`, `=== Relationships (Bot) ===`, etc.
5. **Appearance entries use `==== [[Session NN]] ====`** (four `=`, wiki-linked).
6. **Skip content already on the page.** Diff first.
7. **Mark uncertain quotes with (?).** Auto-captions are noisy.
8. **Every claim needs `<ref>[[Session NN]]</ref>`.**
9. **Every claim needs a YouTube timestamp** where one exists in the transcript.
10. **No em dashes or en dashes.** Never use `--` or `--`. Rephrase with commas, periods, or "to".
11. **Sessions 1-9 are zero-padded:** `[[Session 01]]` through `[[Session 09]]`.
12. **Footer goes at the very end.** After all Bot Conjecture content.
13. **Approval required** before any API call.
14. **Do NOT use `appendtext` API.** It breaks page structure.

---

## Quick Reference

```bash
# Fetch existing page
curl -s "https://evergeennova.fandom.com/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&titles=PAGE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.revisions[0].slots.main["*"]'

# Find mentions in transcripts (no .md extension)
grep -li "CHARACTER" research/sessions/yt-scripts/Session*

# Find mentions in summaries (.md extension)
grep -li "CHARACTER" research/sessions/summaries/Session*.md

# Get YouTube ID from summary
grep "YouTube:" research/sessions/summaries/SessionN.md

# Find timestamps near character mention in transcript
grep -B2 -A2 -i "character_name" research/sessions/yt-scripts/SessionNN

# Check for em/en dashes
grep -cP '[--]' /tmp/page_complete.txt

# Verify line count
wc -l /tmp/page_existing.txt /tmp/page_complete.txt
```
