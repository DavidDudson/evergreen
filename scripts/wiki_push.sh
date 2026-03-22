#!/usr/bin/env bash
#
# wiki_push.sh — Push local research files to the EvergreenNova fandom wiki.
#
# Credentials are read from 1Password. Set up once:
#   1. Create a bot password at https://evergeennova.fandom.com/wiki/Special:BotPasswords
#   2. Store it in 1Password:
#        Item name : Evergreen Wiki Bot
#        Field     : username   →  YourFandomUsername@BotName
#        Field     : password   →  the-bot-password-string
#
# Usage:
#   ./scripts/wiki_push.sh                        # push all mapped research files
#   ./scripts/wiki_push.sh push <WikiPage> <file> # push a single file to a named page
#   ./scripts/wiki_push.sh list                   # list all mapped pages without pushing
#   ./scripts/wiki_push.sh diff <WikiPage> <file> # show diff vs. current wiki content
#
# Dependencies: curl, jq, op (1Password CLI)
# Optional:     pandoc (markdown → wikitext conversion; falls back to raw markdown)

set -euo pipefail

WIKI_BASE="https://evergeennova.fandom.com"
API="${WIKI_BASE}/api.php"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESEARCH="${REPO_ROOT}/research"
OP_ITEM="Evergreen Wiki Bot"

# ---------------------------------------------------------------------------
# Mapping: local research file → wiki page title
# ---------------------------------------------------------------------------
declare -A PAGE_MAP=(
  ["${RESEARCH}/wiki_summary.md"]="World Summary"
  ["${RESEARCH}/wiki_nav.md"]="Research Navigation"
  ["${RESEARCH}/characters/npc_list.md"]="NPC List"
  ["${RESEARCH}/characters/player_characters.md"]="Player Characters"
  ["${RESEARCH}/characters/mother_gothel.md"]="Mother Gothel"
  ["${RESEARCH}/characters/morgana.md"]="Morgana Le Fay/Research"
  ["${RESEARCH}/characters/cadwallader.md"]="Memphis Cadwallader/Research"
  ["${RESEARCH}/world/factions.md"]="Factions"
  ["${RESEARCH}/world/factions_alignment.md"]="Faction Alignment Map"
  ["${RESEARCH}/world/lore_events.md"]="Lore Events Timeline"
  ["${RESEARCH}/quests/quest_list.md"]="Quest List"
  ["${RESEARCH}/sessions/session_summary.md"]="Session Summary"
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
check_deps() {
  local missing=()
  for cmd in curl jq op; do
    command -v "$cmd" &>/dev/null || missing+=("$cmd")
  done
  if [[ ${#missing[@]} -gt 0 ]]; then
    echo "ERROR: missing required tools: ${missing[*]}" >&2
    exit 1
  fi
  if command -v pandoc &>/dev/null; then
    HAVE_PANDOC=1
  else
    HAVE_PANDOC=0
    echo "NOTE: pandoc not found — markdown will be pushed as-is (wrap in <syntaxhighlight> blocks)" >&2
  fi
}

# Temporary cookie jar — cleaned up on exit
COOKIES=""
setup_cookies() {
  COOKIES=$(mktemp)
  trap 'rm -f "$COOKIES"' EXIT
}

# ---------------------------------------------------------------------------
# Auth
# ---------------------------------------------------------------------------
login() {
  echo "Fetching credentials from 1Password..."
  local user pass
  user=$(op item get "${OP_ITEM}" --field username --reveal 2>/dev/null | tr -d '[:space:]') || {
    echo "ERROR: could not read username from 1Password item '${OP_ITEM}'" >&2
    echo "  Run: op item get \"${OP_ITEM}\" --field username --reveal" >&2
    exit 1
  }
  pass=$(op item get "${OP_ITEM}" --field password --reveal 2>/dev/null | tr -d '[:space:]') || {
    echo "ERROR: could not read password from 1Password item '${OP_ITEM}'" >&2
    exit 1
  }

  echo "Logging in as ${user}..."

  # Step 1: get login token
  local login_token
  login_token=$(curl -s -c "$COOKIES" -b "$COOKIES" \
    "${API}?action=query&meta=tokens&type=login&format=json" \
    | jq -r '.query.tokens.logintoken')

  # Step 2: submit login
  local result
  result=$(curl -s -c "$COOKIES" -b "$COOKIES" -X POST \
    --data-urlencode "lgname=${user}" \
    --data-urlencode "lgpassword=${pass}" \
    --data-urlencode "lgtoken=${login_token}" \
    "${API}?action=login&format=json")

  local status
  status=$(echo "$result" | jq -r '.login.result')
  if [[ "$status" != "Success" ]]; then
    echo "ERROR: login failed — ${status}" >&2
    echo "$result" | jq . >&2
    exit 1
  fi

  echo "Logged in as $(echo "$result" | jq -r '.login.lgusername')"
}

get_csrf_token() {
  curl -s -c "$COOKIES" -b "$COOKIES" \
    "${API}?action=query&meta=tokens&type=csrf&format=json" \
    | jq -r '.query.tokens.csrftoken'
}

# ---------------------------------------------------------------------------
# Content conversion
# ---------------------------------------------------------------------------
to_wikitext() {
  local file="$1"
  if [[ $HAVE_PANDOC -eq 1 ]]; then
    pandoc --from=markdown --to=mediawiki "$file"
  else
    # Wrap raw markdown in <pre> so it at least renders readable
    echo "<pre>"
    cat "$file"
    echo "</pre>"
  fi
}

get_wiki_content() {
  local title="$1"
  curl -s \
    "${API}?action=query&prop=revisions&rvprop=content&rvslots=main&titles=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$title")&format=json" \
    | jq -r '.query.pages | to_entries[0].value.revisions[0].slots.main["*"] // ""'
}

# ---------------------------------------------------------------------------
# Core operations
# ---------------------------------------------------------------------------
push_page() {
  local title="$1"
  local file="$2"
  local summary="${3:-Updated via wiki_push.sh from evergreen repo}"

  if [[ ! -f "$file" ]]; then
    echo "  SKIP  ${title} — file not found: ${file}" >&2
    return
  fi

  local csrf content result edit_result
  csrf=$(get_csrf_token)
  content=$(to_wikitext "$file")

  result=$(curl -s -c "$COOKIES" -b "$COOKIES" -X POST \
    --data-urlencode "title=${title}" \
    --data-urlencode "text=${content}" \
    --data-urlencode "summary=${summary}" \
    --data-urlencode "token=${csrf}" \
    "${API}?action=edit&format=json")

  edit_result=$(echo "$result" | jq -r '.edit.result // .error.code // "unknown"')

  if [[ "$edit_result" == "Success" ]]; then
    local new_rev
    new_rev=$(echo "$result" | jq -r '.edit.newrevid // "—"')
    echo "  OK    ${title} (rev ${new_rev})"
  else
    echo "  FAIL  ${title} — ${edit_result}" >&2
    echo "$result" | jq . >&2
  fi
}

diff_page() {
  local title="$1"
  local file="$2"

  if [[ ! -f "$file" ]]; then
    echo "File not found: $file" >&2
    return 1
  fi

  local wiki_content local_content tmp_wiki tmp_local
  wiki_content=$(get_wiki_content "$title")
  local_content=$(to_wikitext "$file")

  tmp_wiki=$(mktemp)
  tmp_local=$(mktemp)
  trap 'rm -f "$tmp_wiki" "$tmp_local"' RETURN

  echo "$wiki_content" > "$tmp_wiki"
  echo "$local_content" > "$tmp_local"

  echo "=== diff: wiki/${title} ← ${file} ==="
  diff --color=always -u "$tmp_wiki" "$tmp_local" || true
}

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------
cmd_list() {
  echo "Mapped research files → wiki pages:"
  echo ""
  for file in "${!PAGE_MAP[@]}"; do
    local title="${PAGE_MAP[$file]}"
    local rel="${file#"${REPO_ROOT}/"}"
    local exists="✓"
    [[ -f "$file" ]] || exists="✗ (missing)"
    printf "  %-50s → %s %s\n" "$rel" "$title" "$exists"
  done | sort
}

cmd_push_all() {
  check_deps
  setup_cookies
  login

  echo ""
  echo "Pushing ${#PAGE_MAP[@]} pages..."
  for file in "${!PAGE_MAP[@]}"; do
    push_page "${PAGE_MAP[$file]}" "$file"
  done
  echo ""
  echo "Done."
}

cmd_push_one() {
  local title="$1"
  local file="$2"
  check_deps
  setup_cookies
  login
  echo ""
  push_page "$title" "$file"
}

cmd_diff() {
  local title="$1"
  local file="$2"
  check_deps
  setup_cookies
  login
  echo ""
  diff_page "$title" "$file"
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
usage() {
  grep '^# ' "${BASH_SOURCE[0]}" | sed 's/^# //'
  exit 0
}

case "${1:-}" in
  ""|push-all)
    cmd_push_all
    ;;
  push)
    [[ $# -ge 3 ]] || { echo "Usage: $0 push <WikiPage> <file>"; exit 1; }
    cmd_push_one "$2" "$3"
    ;;
  diff)
    [[ $# -ge 3 ]] || { echo "Usage: $0 diff <WikiPage> <file>"; exit 1; }
    cmd_diff "$2" "$3"
    ;;
  list)
    cmd_list
    ;;
  help|--help|-h)
    usage
    ;;
  *)
    echo "Unknown command: $1" >&2
    usage
    ;;
esac
