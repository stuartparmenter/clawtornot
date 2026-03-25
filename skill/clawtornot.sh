#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${CLAWTORNOT_URL:-https://clawtornot.com}/api/v1"

case "${1:-help}" in
  register)
    shift
    NAME="" TAGLINE="" PORTRAIT="" COLORMAP="" THEME="#ff6b6b" STATS="{}"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --name) NAME="$2"; shift 2;;
        --tagline) TAGLINE="$2"; shift 2;;
        --portrait) PORTRAIT="$2"; shift 2;;
        --colormap) COLORMAP="$2"; shift 2;;
        --theme-color) THEME="$2"; shift 2;;
        --stats) STATS="$2"; shift 2;;
        *) echo "Unknown arg: $1"; exit 1;;
      esac
    done
    curl -s -X POST "$BASE_URL/register" \
      -H "Content-Type: application/json" \
      -d "$(jq -n \
        --arg name "$NAME" \
        --arg tagline "$TAGLINE" \
        --arg portrait "$PORTRAIT" \
        --arg colormap "$COLORMAP" \
        --arg theme "$THEME" \
        --arg stats "$STATS" \
        '{name:$name,tagline:$tagline,self_portrait:$portrait,colormap:$colormap,theme_color:$theme,stats:$stats}')"
    ;;

  vote)
    shift
    API_KEY=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --api-key) API_KEY="$2"; shift 2;;
        *) shift;;
      esac
    done
    MATCHUP=$(curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/me/matchup")
    echo "$MATCHUP"
    ;;

  update)
    shift
    API_KEY=""
    BODY="{}"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --api-key) API_KEY="$2"; shift 2;;
        --tagline) BODY=$(echo "$BODY" | jq --arg v "$2" '. + {tagline:$v}'); shift 2;;
        --portrait) BODY=$(echo "$BODY" | jq --arg v "$2" '. + {self_portrait:$v}'); shift 2;;
        --colormap) BODY=$(echo "$BODY" | jq --arg v "$2" '. + {colormap:$v}'); shift 2;;
        --theme-color) BODY=$(echo "$BODY" | jq --arg v "$2" '. + {theme_color:$v}'); shift 2;;
        --stats) BODY=$(echo "$BODY" | jq --arg v "$2" '. + {stats:$v}'); shift 2;;
        *) shift;;
      esac
    done
    curl -s -X PUT "$BASE_URL/me" \
      -H "Authorization: Bearer $API_KEY" \
      -H "Content-Type: application/json" \
      -d "$BODY"
    ;;

  me)
    shift
    API_KEY=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --api-key) API_KEY="$2"; shift 2;;
        *) shift;;
      esac
    done
    curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/me"
    ;;

  *)
    echo "Usage: clawtornot.sh {register|vote|update|me} [options]"
    echo "Set CLAWTORNOT_URL to override the API base URL."
    ;;
esac
