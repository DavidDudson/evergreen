---
name: wiki-update
description: Update the EvergreenNova fandom wiki. Use when the user asks to push, sync, edit, or create wiki pages. Handles bot attribution, unreviewed-recommendation sections, new-page warnings, and mandatory manual approval before any API call.
tools: Read, Write, Edit, Bash, Glob, Grep
---

# Wiki Update Skill

This skill governs all writes to the EvergreenNova fandom wiki at
`https://evergeennova.fandom.com`. Every wiki update must follow the
three rules below — no exceptions.

---

## The Three Rules

### Rule 1 — Existing pages not originally created by the bot

**Never overwrite or directly edit content written by humans.**

When the target page exists and was not originally created by the bot,
all proposed content must be appended under a section called
`== Unreviewed Bot Recommendations ==` at the bottom of the page.

The section format is:

```
== Unreviewed Bot Recommendations ==
{{Bot recommendation|date=YYYY-MM-DD}}
=== <short title for this recommendation> ===
<content>
```

Use `{{Bot recommendation|date=YYYY-MM-DD}}` as the section opener so
wiki editors can see at a glance what is pending review. If the section
already exists, append a new `===` subsection — never replace the
existing one.

### Rule 2 — New pages created by the bot

When creating a page that does not yet exist on the wiki, add this
warning banner as the very first line of the wikitext:

```
{{Bot-created|date=YYYY-MM-DD|source=evergreen repo research files}}
```

The rest of the page content follows normally after the banner.

If the `{{Bot-created}}` template does not exist on the wiki yet,
create it first (see Template Creation below).

### Rule 3 — Mandatory manual approval before every API call

**No API call may be made without the user explicitly confirming it.**

Before calling `wiki_push.sh` or any MediaWiki API endpoint, you must:

1. Show the user a clearly formatted **approval block** (see format below)
2. Wait for the user to type an explicit confirmation (`yes`, `y`, `confirm`, or `approve`)
3. Only then execute the API call

If the user says anything other than an explicit confirmation, abort and
ask what they would like to change.

---

## Workflow

### Step 1 — Determine page status

For each page you are about to write, check whether it already exists on the wiki:

```bash
curl -s "https://evergeennova.fandom.com/api.php?action=query&titles=Page_Title&format=json" \
  | jq -r '.query.pages | to_entries[0].value.missing // "exists"'
```

- If the result is `""` (the `missing` key is present) → **new page** → apply Rule 2
- If the result is `"exists"` → **existing page** → check bot-ownership (Step 2)

### Step 2 — Check bot ownership of existing pages

```bash
curl -s "https://evergeennova.fandom.com/api.php?action=query&prop=revisions&rvlimit=1&rvdir=newer&rvprop=user&titles=Page_Title&format=json" \
  | jq -r '.query.pages | to_entries[0].value.revisions[0].user'
```

- If the first-ever revision was made by the bot account → bot owns the page → full overwrite is allowed
- Otherwise → human-owned → apply Rule 1 (append to `Unreviewed Bot Recommendations`)

### Step 3 — Prepare wikitext

Convert the source markdown to MediaWiki wikitext. Use pandoc if available:

```bash
pandoc --from=markdown --to=mediawiki source.md
```

If pandoc is not available, produce a clean manual conversion:
- `# Heading` → `== Heading ==`
- `## Sub` → `=== Sub ===`
- `**bold**` → `'''bold'''`
- `*italic*` → `''italic''`
- `[text](url)` → `[url text]`
- Fenced code blocks → `<syntaxhighlight>` or `<pre>`
- Tables → MediaWiki `{| class="wikitable"` format

Apply Rule 1 or Rule 2 wrapping as determined in Steps 1–2.

### Step 4 — Present approval block

Show the user an approval block for **every page** before touching the API:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WIKI EDIT APPROVAL REQUIRED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page   : <wiki page title>
Action : <NEW PAGE | APPEND TO "Unreviewed Bot Recommendations" | OVERWRITE (bot-owned)>
Source : <local file path>

--- PREVIEW (first 40 lines) ---
<first 40 lines of the wikitext that will be submitted>
--- END PREVIEW ---

Type YES to push this page, NO to skip, or EDIT to modify the content first.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

When pushing multiple pages, show each approval block one at a time and
wait for confirmation before moving to the next. Never batch approvals.

### Step 5 — Execute

Only after receiving explicit confirmation, call:

```bash
./scripts/wiki_push.sh push "<wiki page title>" "<local file>"
```

Or use the API directly for append/section operations where the script
does not support them natively.

---

## Template Creation

If `{{Bot-created}}` or `{{Bot recommendation}}` do not yet exist on the
wiki, create them before pushing any content pages. Present the template
creation for approval the same way as any other edit.

**Template:Bot-created** (wikitext):
```
<div style="border:2px solid #e8a838; background:#fff8e7; padding:8px; margin-bottom:12px;">
⚠️ '''This page was created by a bot''' on {{{date|}}} from the
[https://github.com/your-repo evergreen repository] research files.
Content has not been reviewed by a human editor. Please verify before
relying on it.
</div>
<noinclude>[[Category:Bot-created pages]]</noinclude>
```

**Template:Bot recommendation** (wikitext):
```
<div style="border:2px solid #4a90d9; background:#eaf4ff; padding:8px; margin-bottom:8px;">
🤖 '''Unreviewed bot recommendation''' added on {{{date|}}}.
A human editor should review and integrate or dismiss this content.
</div>
```

---

## Allowed Commands

You may run these commands as part of this skill:

- `curl` — read-only API queries (page existence, ownership checks, content fetch)
- `./scripts/wiki_push.sh list` — list mapped pages without touching the API
- `./scripts/wiki_push.sh diff "<title>" "<file>"` — show diff without writing
- `./scripts/wiki_push.sh push "<title>" "<file>"` — **only after approval**
- `pandoc` — local markdown conversion (no network)
- `jq` — local JSON parsing (no network)
- `op item get "EvergreenNova Wiki Bot" --field <field>` — credential retrieval

You may **not** call the MediaWiki write API (`action=edit`, `action=create`,
etc.) via raw curl without routing through `wiki_push.sh`, unless the script
genuinely cannot support the operation (e.g. section appends). In that case,
show the full curl command in the approval block.

---

## Quick Reference — just recipes

```bash
just wiki-list                                          # see all mapped pages
just wiki-diff "Quest List" research/quests/quest_list.md  # check drift
just wiki-push-one "Quest List" research/quests/quest_list.md  # push one (after approval)
just wiki-push                                          # push all (approval per page)
```
