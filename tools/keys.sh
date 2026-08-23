#!/usr/bin/env bash
# The signing ceremony — both keys, one procedure.
#
#   tools/keys.sh generate     # make both key pairs, print what goes where
#   tools/keys.sh check        # what this repository can prove about them today
#   tools/keys.sh sign <file>  # sign a registry index with the content key
#
# ## Why this is a script and not a page in a document
#
# This repository deleted a design document for being a second source of truth
# that drifts. A key ceremony written as prose is the same shape of thing, with
# a worse failure: prose that has gone stale about *keys* is discovered when
# somebody has already generated one and put it somewhere. A script cannot drift
# from what it does.
#
# ## Two keys, and why not one
#
# ADR 0015. The updater key signs the binary; the content key signs the package
# index. Separate, so that a leak of one is a forged installer OR a forged
# package — never both. The cost is two secrets to look after, and the single
# thing that makes that cost worth paying is that the **procedure** is shared:
# same tool, same storage, same access list, same rotation steps. ADR 0015 says
# what two procedures produce — the one nobody maintains — so this file exists
# to make sure there is only ever one.
#
# `tauri signer` does both. That was not free: until the round that wrote this,
# a signature it produced was one the app refused, because it wraps the whole
# minisign file in base64 and `signing::verify` read only the plain form. The
# app now reads both envelopes and a test holds it with a signature this tool
# really produced (`signing.rs`, `a_signature_from_the_tool_the_updater_
# ceremony_uses_is_read`). Without that, the registry key would have needed a
# second tool, and the shared procedure would have been shared in name only.
#
# ## What this script will not do
#
# It never writes a private key inside the repository, never puts one in a
# command that lands in shell history, and never uploads one. The two acts that
# actually publish — setting the repository secrets and pushing a tag — are
# printed for a person to run, because a key ceremony where a script decides
# when to publish is not a ceremony.
set -uo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Outside the repository, deliberately: a key under the working tree is one
# `git add -A` away from being public, and that is not a mistake anybody gets to
# make twice.
keydir="${STACKVO_KEYDIR:-$HOME/.stackvo-keys}"

bold=$'\033[1m'; dim=$'\033[2m'; red=$'\033[31m'; green=$'\033[32m'; yellow=$'\033[33m'; off=$'\033[0m'

say()  { printf '%s\n' "$*"; }
head_() { printf '\n%s▸ %s%s\n\n' "$bold" "$*" "$off"; }
ok()   { printf '  %s✓%s %s\n' "$green" "$off" "$*"; }
warn() { printf '  %s!%s %s\n' "$yellow" "$off" "$*"; }
bad()  { printf '  %s✗%s %s\n' "$red" "$off" "$*"; }

tauri() { ( cd "$repo" && npx --no-install tauri "$@" ); }

# The second line of a minisign `.pub` file — the one that is not a comment.
# `tauri signer generate` base64-encodes the whole file, so it is peeled first.
pubkey_line() {
  local file="$1"
  base64 -d < "$file" 2>/dev/null | sed -n '2p' | tr -d '\r\n'
}

conf_pubkey() {
  node -p "require('$repo/src-tauri/tauri.conf.json').plugins?.updater?.pubkey ?? ''" 2>/dev/null
}

# ------------------------------------------------------------------ generate

generate() {
  # `tauri signer generate` asks for the password on the terminal and aborts —
  # with a panic, not an error — when there is not one. Caught here so the
  # answer is a sentence rather than a stack trace from somebody else's crate.
  # There is deliberately no unattended mode: a key ceremony with a
  # `--no-password` flag is a ceremony that is eventually run by a script.
  if [ ! -t 0 ]; then
    bad "no terminal. This asks for a password twice, so run it from a shell you are sitting at."
    return 1
  fi

  mkdir -p "$keydir"
  chmod 700 "$keydir"

  head_ "Where the keys will live"
  say "  $keydir"
  say "  ${dim}Outside the repository. Back this directory up before you go further —"
  say "  a lost updater key means no machine already running StackVo can ever be"
  say "  updated again, and there is no recovery for that.${off}"

  for kind in updater registry; do
    local key="$keydir/$kind.key"
    if [ -f "$key" ]; then
      head_ "$kind — already here, left alone"
      say "  $key"
      say "  ${dim}Regenerating would orphan everything the old key signed. Delete it"
      say "  by hand if that is really what you mean.${off}"
      continue
    fi

    head_ "$kind — generating"
    # A password is asked for interactively by the tool. Not passed as an
    # argument: an argument is in the process table and in shell history.
    if ! tauri signer generate -w "$key"; then
      bad "generation failed"
      return 1
    fi
    chmod 600 "$key"
  done

  local updater_pub registry_pub
  updater_pub="$(pubkey_line "$keydir/updater.key.pub")"
  registry_pub="$(pubkey_line "$keydir/registry.key.pub")"

  head_ "1 · The updater's public half → src-tauri/tauri.conf.json"
  say "  plugins.updater.pubkey ="
  say "  $(cat "$keydir/updater.key.pub")"
  say ""
  say "  ${dim}That is the base64 blob, not the line inside it — Tauri reads the"
  say "  whole file.${off}"

  head_ "2 · The registry's public half → src-tauri/src/signing.rs"
  say "  pub const PINNED: &[&str] = &["
  say "      \"$registry_pub\","
  say "  ];"
  say ""
  say "  ${dim}The key LINE here, not the blob — a pinned key is what a person"
  say "  copies out of a .pub file and what a policy file carries.${off}"

  head_ "3 · The private halves → repository secrets"
  say "  ${dim}Run these yourself. They are not run for you: a script that publishes"
  say "  a key is a script that decides when the ceremony happened.${off}"
  say ""
  say "  gh secret set TAURI_SIGNING_PRIVATE_KEY < $keydir/updater.key"
  say "  gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
  say ""
  say "  ${dim}The registry key gets NO repository secret. It signs an index in the"
  say "  packages repository, by hand, from this machine — see \`keys.sh sign\`."
  say "  A content key sitting in CI is a content key every workflow can reach.${off}"

  head_ "4 · Then"
  say "  tools/keys.sh check"
  say "  ${dim}and when that is clean, the release: docs are in tools/release.sh${off}"
}

# --------------------------------------------------------------------- check

check() {
  local problems=0

  head_ "The updater key"
  local conf_pub
  conf_pub="$(conf_pubkey)"
  if [ -z "$conf_pub" ]; then
    bad "tauri.conf.json carries no plugins.updater.pubkey — every update would be accepted unverified"
    problems=$((problems + 1))
  else
    ok "tauri.conf.json carries a public key"
    if [ -f "$keydir/updater.key.pub" ]; then
      if [ "$conf_pub" = "$(tr -d '\r\n' < "$keydir/updater.key.pub")" ]; then
        ok "and it is the public half of $keydir/updater.key"
      else
        bad "but it is NOT the public half of $keydir/updater.key — the private key you hold cannot sign for this build"
        problems=$((problems + 1))
      fi
    else
      warn "no local $keydir/updater.key — cannot tell whether the private half still exists anywhere"
    fi
  fi

  head_ "The content key"
  local pinned
  pinned="$(grep -c '^\s*"RW' "$repo/src-tauri/src/signing.rs" || true)"
  if [ "$pinned" = "0" ]; then
    warn "signing::PINNED is empty — a signed refresh is refused, which is the honest state and not the finished one"
  else
    ok "signing::PINNED carries $pinned key(s)"
  fi
  if [ -f "$keydir/registry.key" ]; then
    ok "a content key exists at $keydir/registry.key"
  else
    warn "no $keydir/registry.key — run: tools/keys.sh generate"
  fi

  head_ "The two are not the same key"
  local updater_line registry_line
  updater_line="$( [ -n "$conf_pub" ] && printf '%s' "$conf_pub" | base64 -d 2>/dev/null | sed -n '2p' | tr -d '\r\n' )"
  registry_line="$(grep -o '"RW[A-Za-z0-9+/=]*"' "$repo/src-tauri/src/signing.rs" | head -1 | tr -d '"')"
  if [ -n "$updater_line" ] && [ -n "$registry_line" ] && [ "$updater_line" = "$registry_line" ]; then
    bad "the updater and the registry are pinned to ONE key — a single leak forges both a binary and a package (ADR 0015)"
    problems=$((problems + 1))
  else
    ok "separate, or not yet both set"
  fi

  head_ "Nothing secret is in the tree"
  # Asks whether a file **is** a key, not whether it mentions one. Grepping for
  # the header was the first version and it reported this script and its own
  # Rust gate — both merely contain the words, one of them as the needle. The
  # shape is what tells them apart: a secret key is a comment line and a base64
  # line, and `tauri signer` writes that pair base64-encoded again onto a single
  # line, which is the form one would actually be committed in.
  local leaked=""
  while IFS= read -r file; do
    [ -f "$repo/$file" ] || continue
    # Only files small enough and short enough to be a key at all. Anything with
    # prose in it fails this before it is ever decoded.
    [ "$(wc -c <"$repo/$file")" -le 4096 ] || continue
    [ "$(wc -l <"$repo/$file")" -le 3 ] || continue
    local head
    head="$(head -c 4096 "$repo/$file" | base64 -d 2>/dev/null | head -1)"
    case "$head" in
      *"encrypted secret key"*) leaked="$leaked $file" ;;
      *) head="$(head -1 "$repo/$file")"
         case "$head" in *"encrypted secret key"*) leaked="$leaked $file" ;; esac ;;
    esac
  done < <(git -C "$repo" ls-files)

  if [ -n "$leaked" ]; then
    bad "a PRIVATE key is committed:$leaked — rotate it, it is public the moment it is pushed"
    problems=$((problems + 1))
  else
    ok "no private key is committed"
  fi

  head_ "The release path"
  if [ -z "$(git -C "$repo" tag --list 'v*')" ]; then
    warn "no v* tag has ever been pushed — the updater endpoint 404s until the first release exists"
  else
    ok "a v* tag exists"
  fi

  printf '\n'
  if [ "$problems" -gt 0 ]; then
    printf '%s%s problem(s).%s The warnings above are states, not faults.\n' "$red" "$problems" "$off"
    return 1
  fi
  printf '%sNothing wrong.%s Anything marked ! is a step nobody has taken yet.\n' "$green" "$off"
}

# ---------------------------------------------------------------------- sign

sign() {
  local file="${1:-}"
  if [ -z "$file" ] || [ ! -f "$file" ]; then
    bad "usage: tools/keys.sh sign <registry.json>"
    return 1
  fi
  if [ ! -f "$keydir/registry.key" ]; then
    bad "no content key at $keydir/registry.key — run: tools/keys.sh generate"
    return 1
  fi

  # Written next to the file as `.minisig`, which is the name `market::refresh`
  # fetches. `tauri signer` names its output `.sig`, so it is moved rather than
  # left for somebody to rename correctly under time pressure.
  tauri signer sign -f "$keydir/registry.key" "$file" >/dev/null || return 1
  mv "$file.sig" "$file.minisig"
  ok "signed → $file.minisig"
  say "  ${dim}Publish it beside registry.json. The app fetches both and checks the"
  say "  signature before it parses a single field of the index.${off}"
}

case "${1:-}" in
  generate) generate ;;
  check)    check ;;
  sign)     shift; sign "$@" ;;
  *)
    say "usage: tools/keys.sh {generate|check|sign <file>}"
    say ""
    say "  generate  both key pairs, and what to do with each half"
    say "  check     what this repository can prove about them today"
    say "  sign      sign a registry index with the content key"
    exit 1
    ;;
esac
