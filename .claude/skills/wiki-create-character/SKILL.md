---
name: wiki-create-character
description: Create a new character page on the EvergreenNova fandom wiki from session transcripts and research files. Use when the user asks to create a wiki page for a character that doesn't have one yet.
tools: Read, Write, Edit, Bash, Glob, Grep, Agent, WebFetch
---

# Wiki Create Character Skill

Creates a **new** character page on the EvergreenNova fandom wiki, sourced
from raw session transcripts, session summaries, and research files. Uses
the `{{Bot-created}}` banner (Rule 2 from wiki-update skill).

---

## Inputs

The user provides:
- **Character name** (required) — e.g. `Kristoff Marlo`, `Titus Skye`
- **Session range** (optional) — defaults to all available sessions

---

## Workflow

### Step 1 — Confirm the page does NOT exist

```bash
curl -s "https://evergeennova.fandom.com/api.php?action=query&titles=PAGE_TITLE&format=json" \
  | jq -r '.query.pages | to_entries[0].value.missing // "exists"'
```

If the page already exists, abort and suggest using `/wiki-enrich` instead.

### Step 2 — Gather all information about the character

Search multiple sources:

```bash
# Research files
grep -rli "CHARACTER" research/characters/ research/world/ research/quests/

# Session summaries
grep -li "CHARACTER" research/sessions/summaries/Session*.md

# Raw transcripts (use garbled name variants too)
grep -li "CHARACTER" research/sessions/yt-scripts/Session*
```

Read the NPC list entry, player characters entry, character alignments entry,
and all relevant session summaries. Dig into raw transcripts for quotes and
timestamps.

### Step 3 — Extract the YouTube video IDs

Each summary has a YouTube link:
```
**YouTube:** [Session N](https://www.youtube.com/watch?v=VIDEO_ID)
```

Raw transcripts have timestamp links:
```
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]
```

### Step 4 — Build the page

Use the Queen Maeve page as the structural template. A character page has:

```mediawiki
{{Bot-created|date=YYYY-MM-DD|source=evergreen repo session transcripts}}
{{Evergreen_NPC
|title1=CHARACTER NAME
|also_known_as=ALIASES
|ancestry=RACE/ANCESTRY
|affiliation=[[FACTION]]
|place=LOCATION
|role/office=ROLE
|languages=Common
|gender/presentation=GENDER
|eyes=EYE_COLOUR
|status=STATUS
|creature_type=TYPE
|relatives=RELATIVES (wiki-linked)
|hair=HAIR
}}
'''CHARACTER NAME''' is SHORT_DESCRIPTION.

== Description ==
1-2 paragraphs describing who they are, what they look like, and their
role in the world. Write in present tense for living characters.

=== Appearance ===
Physical description if known from transcripts.

=== History ===
Narrative history in chronological order. Reference sessions:
<ref>[[Session N]]</ref>

Use sub-headings for major events:
==== Event Name ====
Description.

== Relationships ==
=== [[Character Name]] ===
Description of the relationship. <ref>[[Session N]]</ref>

== Quotes ==
:"''Quote text here.''" — [[Session N]] [https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (HH:MM:SS)]

== Birth / Death ==
Only include if relevant. Birth details, death details, resurrection if
applicable. Omit section entirely if nothing is known.

== Appearances in the Campaign ==
=== [[Session N]] ===
Brief narrative of what happened with this character in this session.
[https://www.youtube.com/watch?v=VIDEO_ID&t=SECONDS (timestamp)] for key
moments.

{{Template:Nav-Characters}}
[[Category:Characters]]
== References ==
<references />
```

### Step 5 — Infobox field guide

Fill in what you can, leave blank what you can't. The `{{Evergreen_NPC}}`
template fields:

| Field | Description | Example |
|---|---|---|
| title1 | Display name | Kristoff Marlo |
| image1 | Image filename (leave blank if none) | |
| also_known_as | Aliases | The Sewer Wizard |
| ancestry | Race/species | Human |
| affiliation | Faction (wiki-linked) | [[Order of the Eleventh Eye]] |
| place | Primary location | [[Salasarglumm]] |
| role/office | Title or role | Fugitive Wizard |
| languages | Languages spoken | Common |
| gender/presentation | Gender | Male |
| eyes | Eye colour | Unknown |
| status | Alive/Dead/Unknown | Alive |
| creature_type | Creature type | Humanoid |
| relatives | Wiki-linked relatives | |
| height | Height if known | |
| hair | Hair description | |

### Step 6 — Quality checks

Before presenting for approval:

1. **All claims have session references** — every factual statement ends with `<ref>[[Session N]]</ref>`
2. **Quotes are attributed** — speaker and session identified
3. **Wiki links used** — character names, locations, factions, items all linked with `[[Name]]`
4. **YouTube timestamps** included where available for key moments
5. **Tone matches existing wiki** — past tense narrative, encyclopaedic, neutral
6. **No duplicate content** — if the character appears on another page, don't repeat the same info verbatim
7. **Bot-created banner** is the first line

### Step 7 — Present for approval

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WIKI PAGE CREATION APPROVAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page     : <wiki page title>
Action   : NEW PAGE (Bot-created)
Sections : Description, History, Relationships, Quotes, Appearances
Sessions : <session numbers referenced>
Length   : ~<word count> words

--- PREVIEW (first 60 lines) ---
<wikitext preview>
--- END PREVIEW ---

Type YES to create, NO to skip, or EDIT to modify.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Step 8 — Push

Save the wikitext to a local file, then push. **The file MUST be raw
mediawiki wikitext** (with `==` headings, `[[links]]`, `{{templates}}`).
Do NOT write markdown — `wiki_push.sh` auto-detects wikitext and passes
it through without conversion.

```bash
# Save to temp file — use heredoc with single-quoted delimiter to prevent
# shell expansion of {{ }} and [[ ]]
cat > /tmp/wiki_page.txt << 'EOF'
<wikitext content>
EOF

# Push using wiki_push.sh
./scripts/wiki_push.sh push "<Page Title>" /tmp/wiki_page.txt
```

**Important:** The file must contain `{{` or `==` headings on their own
lines for `wiki_push.sh` to detect it as wikitext. If the file looks like
markdown, pandoc will convert it and mangle the mediawiki markup.

### Step 9 — Update local research files

After the wiki page is created, update the local research files to keep
them in sync. This is **mandatory** — do not skip.

#### 9a. NPC List (`research/characters/npc_list.md`)

Check if the character is already in the NPC list. If not, add a row to
the appropriate table in the Session-Introduced NPCs section (Allies or
Villains). Include a wiki link to the new page.

Format (Allies table):
```markdown
| [**Name**](https://evergeennova.fandom.com/wiki/Page_Title) | SN | Status | alignment | Role description. |
```

If the character already exists in the NPC list but without a wiki link,
add the link.

#### 9b. Character Alignments (`research/characters/character_alignments.md`)

Add the character to the correct faction section (Greenwoods, Darkwoods,
or Cities) with a wiki link. Place them in the appropriate sub-section
(Allies, Villains, Sorcerers, etc.).

#### 9c. Push updated research files to wiki

```bash
./scripts/wiki_push.sh push "NPC List" "research/characters/npc_list.md"
./scripts/wiki_push.sh push "Character Alignments" "research/characters/character_alignments.md"
```

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

---

## Rules

1. **Page must NOT already exist.** Check first. If it exists, use `/wiki-enrich`.
2. **`{{Bot-created}}` banner required** as the first line.
3. **All content needs session references** — `<ref>[[Session N]]</ref>`.
4. **Include YouTube timestamps** for quotes and key moments.
5. **Use wiki links** for all characters, locations, factions, items.
6. **Write in the wiki's existing tone** — past tense, narrative, encyclopaedic.
7. **Approval required** before any API call.
8. **One character at a time.**
9. **Include `{{Template:Nav-Characters}}` and `[[Category:Characters]]`** at the bottom.

---

## Characters Without Wiki Pages (Priority List)

These characters from the research files have no wiki page yet:

| Character | Sessions | Priority |
|---|---|---|
| Kristoff Marlo | S24-31 | High (major ally) |
| Governor Eustus Moro | S25-31 | High (major ally) |
| Tristan Iran | S25-31 | Medium |
| Galen Aran | S26-31 | Medium |
| Mother Ren | S26 | Medium |
| Titus Skye | S17 | Medium |
| Lord Edmund | S13-16 | Medium |
| High Abbot Jor | S21 | Low |
| Amelia (OXIE) | S34, S41 | Low |
| Barnabas Ragdoll | S61 | Medium (Wizard of Oz) |
| The Conductor | S7 | Low |
| Pyromancer Ken | S6 | Low |
