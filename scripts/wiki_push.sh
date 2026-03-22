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

# Credential cache — fetch from 1Password once per session
CRED_CACHE_DIR="${XDG_RUNTIME_DIR:-/tmp}/wiki-push-creds-$(id -u)"
CRED_CACHE_USER="${CRED_CACHE_DIR}/username"
CRED_CACHE_PASS="${CRED_CACHE_DIR}/password"
CRED_CACHE_TTL=3600  # seconds before cache expires

# ---------------------------------------------------------------------------
# Mapping: local research file → wiki page title
# ---------------------------------------------------------------------------
declare -A PAGE_MAP=(
  ["${RESEARCH}/wiki_summary.md"]="World Summary"
  ["${RESEARCH}/wiki_nav.md"]="Research Navigation"
  ["${RESEARCH}/characters/npc_list.md"]="NPC List"
  ["${RESEARCH}/characters/player_characters.md"]="Player Characters"
  ["${RESEARCH}/characters/character_alignments.md"]="Character Alignments"
  ["${RESEARCH}/characters/mother_gothel.md"]="Mother Gothel"
  ["${RESEARCH}/characters/morgana.md"]="Morgana Le Fay/Research"
  ["${RESEARCH}/characters/cadwallader.md"]="Memphis Cadwallader/Research"
  ["${RESEARCH}/world/factions.md"]="Factions"
  ["${RESEARCH}/world/factions_alignment.md"]="Faction Alignment Map"
  ["${RESEARCH}/world/lore_events.md"]="Lore Events Timeline"
  ["${RESEARCH}/quests/quest_list.md"]="Quest List"
  ["${RESEARCH}/sessions/session_summary.md"]="Session Summary"
  ["${RESEARCH}/sessions/hero_questions.md"]="Hero Questions"
  ["${RESEARCH}/sessions/yt-scripts/Session1.md"]="Session 1/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session2.md"]="Session 2/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session3.md"]="Session 3/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session4.md"]="Session 4/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session5.md"]="Session 5/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session6.md"]="Session 6/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session7.md"]="Session 7/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session8.md"]="Session 8/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session9.md"]="Session 9/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session10.md"]="Session 10/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session11.md"]="Session 11/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session12.md"]="Session 12/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session13.md"]="Session 13/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session14.md"]="Session 14/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session15.md"]="Session 15/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session16.md"]="Session 16/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session17.md"]="Session 17/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session18.md"]="Session 18/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session19.md"]="Session 19/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session20.md"]="Session 20/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session21.md"]="Session 21/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session22.md"]="Session 22/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session23.md"]="Session 23/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session25.md"]="Session 25/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session26.md"]="Session 26/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session27.md"]="Session 27/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session28.md"]="Session 28/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session29.md"]="Session 29/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session30.md"]="Session 30/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session31.md"]="Session 31/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session32.md"]="Session 32/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session33.md"]="Session 33/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session34.md"]="Session 34/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session35.md"]="Session 35/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session36.md"]="Session 36/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session37.md"]="Session 37/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session38.md"]="Session 38/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session39.md"]="Session 39/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session41.md"]="Session 41/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session42.md"]="Session 42/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session43.md"]="Session 43/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session44.md"]="Session 44/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session45.md"]="Session 45/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session46.md"]="Session 46/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session47.md"]="Session 47/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session48.md"]="Session 48/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session49.md"]="Session 49/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session50.md"]="Session 50/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session51.md"]="Session 51/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session52.md"]="Session 52/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session53.md"]="Session 53/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session54.md"]="Session 54/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session55.md"]="Session 55/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session56.md"]="Session 56/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session57.md"]="Session 57/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session58.md"]="Session 58/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session59.md"]="Session 59/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session60.md"]="Session 60/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session61.md"]="Session 61/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session62.md"]="Session 62/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session63.md"]="Session 63/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session64.md"]="Session 64/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session65.md"]="Session 65/Transcript"
  ["${RESEARCH}/sessions/yt-scripts/Session66.md"]="Session 66/Transcript"
  ["${RESEARCH}/sessions/summaries/Session1.md"]="Session 1/Summary"
  ["${RESEARCH}/sessions/summaries/Session2.md"]="Session 2/Summary"
  ["${RESEARCH}/sessions/summaries/Session3.md"]="Session 3/Summary"
  ["${RESEARCH}/sessions/summaries/Session4.md"]="Session 4/Summary"
  ["${RESEARCH}/sessions/summaries/Session5.md"]="Session 5/Summary"
  ["${RESEARCH}/sessions/summaries/Session6.md"]="Session 6/Summary"
  ["${RESEARCH}/sessions/summaries/Session7.md"]="Session 7/Summary"
  ["${RESEARCH}/sessions/summaries/Session8.md"]="Session 8/Summary"
  ["${RESEARCH}/sessions/summaries/Session9.md"]="Session 9/Summary"
  ["${RESEARCH}/sessions/summaries/Session10.md"]="Session 10/Summary"
  ["${RESEARCH}/sessions/summaries/Session11.md"]="Session 11/Summary"
  ["${RESEARCH}/sessions/summaries/Session12.md"]="Session 12/Summary"
  ["${RESEARCH}/sessions/summaries/Session13.md"]="Session 13/Summary"
  ["${RESEARCH}/sessions/summaries/Session14.md"]="Session 14/Summary"
  ["${RESEARCH}/sessions/summaries/Session15.md"]="Session 15/Summary"
  ["${RESEARCH}/sessions/summaries/Session16.md"]="Session 16/Summary"
  ["${RESEARCH}/sessions/summaries/Session17.md"]="Session 17/Summary"
  ["${RESEARCH}/sessions/summaries/Session18.md"]="Session 18/Summary"
  ["${RESEARCH}/sessions/summaries/Session19.md"]="Session 19/Summary"
  ["${RESEARCH}/sessions/summaries/Session20.md"]="Session 20/Summary"
  ["${RESEARCH}/sessions/summaries/Session21.md"]="Session 21/Summary"
  ["${RESEARCH}/sessions/summaries/Session22.md"]="Session 22/Summary"
  ["${RESEARCH}/sessions/summaries/Session23.md"]="Session 23/Summary"
  ["${RESEARCH}/sessions/summaries/Session25.md"]="Session 25/Summary"
  ["${RESEARCH}/sessions/summaries/Session26.md"]="Session 26/Summary"
  ["${RESEARCH}/sessions/summaries/Session27.md"]="Session 27/Summary"
  ["${RESEARCH}/sessions/summaries/Session28.md"]="Session 28/Summary"
  ["${RESEARCH}/sessions/summaries/Session29.md"]="Session 29/Summary"
  ["${RESEARCH}/sessions/summaries/Session30.md"]="Session 30/Summary"
  ["${RESEARCH}/sessions/summaries/Session31.md"]="Session 31/Summary"
  ["${RESEARCH}/sessions/summaries/Session32.md"]="Session 32/Summary"
  ["${RESEARCH}/sessions/summaries/Session33.md"]="Session 33/Summary"
  ["${RESEARCH}/sessions/summaries/Session34.md"]="Session 34/Summary"
  ["${RESEARCH}/sessions/summaries/Session35.md"]="Session 35/Summary"
  ["${RESEARCH}/sessions/summaries/Session36.md"]="Session 36/Summary"
  ["${RESEARCH}/sessions/summaries/Session37.md"]="Session 37/Summary"
  ["${RESEARCH}/sessions/summaries/Session38.md"]="Session 38/Summary"
  ["${RESEARCH}/sessions/summaries/Session39.md"]="Session 39/Summary"
  ["${RESEARCH}/sessions/summaries/Session41.md"]="Session 41/Summary"
  ["${RESEARCH}/sessions/summaries/Session42.md"]="Session 42/Summary"
  ["${RESEARCH}/sessions/summaries/Session43.md"]="Session 43/Summary"
  ["${RESEARCH}/sessions/summaries/Session44.md"]="Session 44/Summary"
  ["${RESEARCH}/sessions/summaries/Session45.md"]="Session 45/Summary"
  ["${RESEARCH}/sessions/summaries/Session46.md"]="Session 46/Summary"
  ["${RESEARCH}/sessions/summaries/Session47.md"]="Session 47/Summary"
  ["${RESEARCH}/sessions/summaries/Session48.md"]="Session 48/Summary"
  ["${RESEARCH}/sessions/summaries/Session49.md"]="Session 49/Summary"
  ["${RESEARCH}/sessions/summaries/Session50.md"]="Session 50/Summary"
  ["${RESEARCH}/sessions/summaries/Session51.md"]="Session 51/Summary"
  ["${RESEARCH}/sessions/summaries/Session52.md"]="Session 52/Summary"
  ["${RESEARCH}/sessions/summaries/Session53.md"]="Session 53/Summary"
  ["${RESEARCH}/sessions/summaries/Session54.md"]="Session 54/Summary"
  ["${RESEARCH}/sessions/summaries/Session55.md"]="Session 55/Summary"
  ["${RESEARCH}/sessions/summaries/Session56.md"]="Session 56/Summary"
  ["${RESEARCH}/sessions/summaries/Session57.md"]="Session 57/Summary"
  ["${RESEARCH}/sessions/summaries/Session58.md"]="Session 58/Summary"
  ["${RESEARCH}/sessions/summaries/Session59.md"]="Session 59/Summary"
  ["${RESEARCH}/sessions/summaries/Session60.md"]="Session 60/Summary"
  ["${RESEARCH}/sessions/summaries/Session61.md"]="Session 61/Summary"
  ["${RESEARCH}/sessions/summaries/Session62.md"]="Session 62/Summary"
  ["${RESEARCH}/sessions/summaries/Session63.md"]="Session 63/Summary"
  ["${RESEARCH}/sessions/summaries/Session64.md"]="Session 64/Summary"
  ["${RESEARCH}/sessions/summaries/Session65.md"]="Session 65/Summary"
  ["${RESEARCH}/sessions/summaries/Session66.md"]="Session 66/Summary"
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
fetch_credentials() {
  # Returns cached credentials if fresh, otherwise fetches from 1Password
  local user pass

  if [[ -f "$CRED_CACHE_USER" && -f "$CRED_CACHE_PASS" ]]; then
    local age
    age=$(( $(date +%s) - $(stat -c %Y "$CRED_CACHE_USER" 2>/dev/null || stat -f %m "$CRED_CACHE_USER" 2>/dev/null || echo 0) ))
    if [[ $age -lt $CRED_CACHE_TTL ]]; then
      WIKI_USER=$(cat "$CRED_CACHE_USER")
      WIKI_PASS=$(cat "$CRED_CACHE_PASS")
      return 0
    fi
  fi

  echo "Fetching credentials from 1Password..."
  user=$(op item get "${OP_ITEM}" --field username --reveal 2>/dev/null | tr -d '[:space:]') || {
    echo "ERROR: could not read username from 1Password item '${OP_ITEM}'" >&2
    echo "  Run: op item get \"${OP_ITEM}\" --field username --reveal" >&2
    exit 1
  }
  pass=$(op item get "${OP_ITEM}" --field password --reveal 2>/dev/null | tr -d '[:space:]') || {
    echo "ERROR: could not read password from 1Password item '${OP_ITEM}'" >&2
    exit 1
  }

  # Cache credentials in a restricted directory
  mkdir -p "$CRED_CACHE_DIR"
  chmod 700 "$CRED_CACHE_DIR"
  printf '%s' "$user" > "$CRED_CACHE_USER"
  printf '%s' "$pass" > "$CRED_CACHE_PASS"
  chmod 600 "$CRED_CACHE_USER" "$CRED_CACHE_PASS"

  WIKI_USER="$user"
  WIKI_PASS="$pass"
}

login() {
  fetch_credentials

  echo "Logging in as ${WIKI_USER}..."

  # Step 1: get login token
  local login_token
  login_token=$(curl -s -c "$COOKIES" -b "$COOKIES" \
    "${API}?action=query&meta=tokens&type=login&format=json" \
    | jq -r '.query.tokens.logintoken')

  # Step 2: submit login
  local result
  result=$(curl -s -c "$COOKIES" -b "$COOKIES" -X POST \
    --data-urlencode "lgname=${WIKI_USER}" \
    --data-urlencode "lgpassword=${WIKI_PASS}" \
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

  local csrf result edit_result tmp_content
  csrf=$(get_csrf_token)
  tmp_content=$(mktemp)
  to_wikitext "$file" > "$tmp_content"

  result=$(curl -s -c "$COOKIES" -b "$COOKIES" -X POST \
    --data-urlencode "title=${title}" \
    --data-urlencode "text@${tmp_content}" \
    --data-urlencode "summary=${summary}" \
    --data-urlencode "token=${csrf}" \
    "${API}?action=edit&format=json")
  rm -f "$tmp_content"

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
