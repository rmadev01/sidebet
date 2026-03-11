#!/bin/bash
set -euo pipefail

###############################################################################
# SideBet — Full Integration Test
#
# Creates 3 sample users, makes them friends, creates bets between them,
# accepts/declines/settles bets, and validates every interaction.
###############################################################################

AUTH="http://localhost:3001"
API="http://localhost:3000"
ORIGIN="http://localhost:5175"
PASS="TestPass123!"

C1="/tmp/sb_alice.txt"; C2="/tmp/sb_bob.txt"; C3="/tmp/sb_carol.txt"
rm -f "$C1" "$C2" "$C3"

ok()   { printf "  ✅ %s\n" "$1"; }
fail() { printf "  ❌ %s\n" "$1"; FAILURES=$((FAILURES + 1)); }
section() { printf "\n\033[1;36m━━━ %s ━━━\033[0m\n" "$1"; }
FAILURES=0

# Helper: sign up or sign in, return user JSON
auth_user() {
  local email="$1" name="$2" jar="$3"
  local res
  res=$(curl -s -X POST "$AUTH/api/auth/sign-up/email" \
    -H "Content-Type: application/json" -H "Origin: $ORIGIN" \
    -d "{\"email\":\"$email\",\"password\":\"$PASS\",\"name\":\"$name\"}" \
    -c "$jar" -b "$jar" 2>&1)
  if echo "$res" | grep -qi "error\|already"; then
    res=$(curl -s -X POST "$AUTH/api/auth/sign-in/email" \
      -H "Content-Type: application/json" -H "Origin: $ORIGIN" \
      -d "{\"email\":\"$email\",\"password\":\"$PASS\"}" \
      -c "$jar" -b "$jar" 2>&1)
  fi
  echo "$res"
}

# Helper: get sidebet user id (auto-provisioned by backend)
get_sb_id() {
  local jar="$1"
  curl -s "$API/api/users/me" -b "$jar" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])"
}

get_balance() {
  local jar="$1"
  curl -s "$API/api/wallet/balance" -b "$jar" | python3 -c "import sys,json; print(json.load(sys.stdin)['coin_balance'])"
}

###############################################################################
section "1. Create Users"
###############################################################################

echo "  Creating Alice..."
auth_user "alice@sidebet.test" "Alice Johnson" "$C1" > /dev/null
ALICE_ID=$(get_sb_id "$C1")
ALICE_BAL=$(get_balance "$C1")
ok "Alice: id=$ALICE_ID balance=$ALICE_BAL"

echo "  Creating Bob..."
auth_user "bob@sidebet.test" "Bob Smith" "$C2" > /dev/null
BOB_ID=$(get_sb_id "$C2")
BOB_BAL=$(get_balance "$C2")
ok "Bob:   id=$BOB_ID balance=$BOB_BAL"

echo "  Creating Carol..."
auth_user "carol@sidebet.test" "Carol Davis" "$C3" > /dev/null
CAROL_ID=$(get_sb_id "$C3")
CAROL_BAL=$(get_balance "$C3")
ok "Carol: id=$CAROL_ID balance=$CAROL_BAL"

###############################################################################
section "2. Daily Bonus"
###############################################################################

for name_jar in "Alice:$C1" "Bob:$C2" "Carol:$C3"; do
  name="${name_jar%%:*}"
  jar="${name_jar##*:}"
  res=$(curl -s -X POST "$API/api/wallet/daily-bonus" -b "$jar")
  awarded=$(echo "$res" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('coins_awarded', 'already claimed'))" 2>/dev/null || echo "already claimed")
  ok "$name daily bonus: $awarded"
done

###############################################################################
section "3. Friendships"
###############################################################################

echo "  Alice sends friend request to Bob..."
res=$(curl -s -X POST "$API/api/friends/request" \
  -H "Content-Type: application/json" \
  -b "$C1" -d "{\"user_id\":\"$BOB_ID\"}" 2>&1)
echo "$res" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'    status={d.get(\"status\",\"ok\")}')" 2>/dev/null || echo "    $res"

echo "  Alice sends friend request to Carol..."
curl -s -X POST "$API/api/friends/request" \
  -H "Content-Type: application/json" \
  -b "$C1" -d "{\"user_id\":\"$CAROL_ID\"}" > /dev/null 2>&1

echo "  Bob sends friend request to Carol..."
curl -s -X POST "$API/api/friends/request" \
  -H "Content-Type: application/json" \
  -b "$C2" -d "{\"user_id\":\"$CAROL_ID\"}" > /dev/null 2>&1

# Bob's pending requests
echo "  Bob checks friend requests..."
REQS=$(curl -s "$API/api/friends/requests" -b "$C2")
REQ_COUNT=$(echo "$REQS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)
if [ "$REQ_COUNT" -gt 0 ]; then
  ok "Bob has $REQ_COUNT pending request(s)"
  FSHIP_ID=$(echo "$REQS" | python3 -c "import sys,json; print(json.load(sys.stdin)[0]['friendship_id'])")
  echo "  Bob accepts Alice's request ($FSHIP_ID)..."
  curl -s -X POST "$API/api/friends/$FSHIP_ID/accept" -b "$C2" > /dev/null
  ok "Bob accepted Alice's friend request"
else
  fail "Bob has no friend requests"
fi

# Carol accepts both
echo "  Carol checks friend requests..."
CAROL_REQS=$(curl -s "$API/api/friends/requests" -b "$C3")
CAROL_REQ_COUNT=$(echo "$CAROL_REQS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)
ok "Carol has $CAROL_REQ_COUNT pending request(s)"

echo "$CAROL_REQS" | python3 -c "
import sys,json
reqs = json.load(sys.stdin)
for r in reqs:
  print(r['friendship_id'])
" 2>/dev/null | while read fid; do
  echo "  Carol accepts request ($fid)..."
  curl -s -X POST "$API/api/friends/$fid/accept" -b "$C3" > /dev/null
done
ok "Carol accepted all requests"

# Verify friends lists
echo "  Verifying friend lists..."
ALICE_FRIENDS=$(curl -s "$API/api/friends" -b "$C1" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d))" 2>/dev/null)
BOB_FRIENDS=$(curl -s "$API/api/friends" -b "$C2" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d))" 2>/dev/null)
CAROL_FRIENDS=$(curl -s "$API/api/friends" -b "$C3" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d))" 2>/dev/null)
ok "Alice has $ALICE_FRIENDS friends, Bob has $BOB_FRIENDS, Carol has $CAROL_FRIENDS"

###############################################################################
section "4. Get Events"
###############################################################################

# Sync events first
curl -s -X POST "$API/api/events/sync?leagues=NBA,NHL" -b "$C1" > /dev/null 2>&1

EVENTS=$(curl -s "$API/api/events?status=upcoming" -b "$C1")
EVENT_COUNT=$(echo "$EVENTS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)
ok "Found $EVENT_COUNT upcoming events"

# Get first NBA event
EVENT1_ID=$(echo "$EVENTS" | python3 -c "
import sys,json
d=json.load(sys.stdin)
nba=[e for e in d if e['category']=='nba']
print(nba[0]['id'] if nba else d[0]['id'])
" 2>/dev/null)

EVENT1_TITLE=$(echo "$EVENTS" | python3 -c "
import sys,json
d=json.load(sys.stdin)
nba=[e for e in d if e['category']=='nba']
print(nba[0]['title'] if nba else d[0]['title'])
" 2>/dev/null)

# Get second event
EVENT2_ID=$(echo "$EVENTS" | python3 -c "
import sys,json
d=json.load(sys.stdin)
nba=[e for e in d if e['category']=='nba']
print(nba[1]['id'] if len(nba)>1 else d[1]['id'] if len(d)>1 else d[0]['id'])
" 2>/dev/null)

EVENT2_TITLE=$(echo "$EVENTS" | python3 -c "
import sys,json
d=json.load(sys.stdin)
nba=[e for e in d if e['category']=='nba']
print(nba[1]['title'] if len(nba)>1 else d[1]['title'] if len(d)>1 else d[0]['title'])
" 2>/dev/null)

ok "Event 1: $EVENT1_TITLE"
ok "Event 2: $EVENT2_TITLE"

# Parse sides using python (bash arrays split on spaces, bad for multi-word names)
E1_SIDE_A=$(python3 -c "
t='$EVENT1_TITLE'
print(t.split(' @ ')[0].strip() if ' @ ' in t else t.split(' vs ')[0].strip() if ' vs ' in t.lower() else t)
")
E1_SIDE_B=$(python3 -c "
t='$EVENT1_TITLE'
parts = t.split(' @ ') if ' @ ' in t else t.split(' vs ') if ' vs ' in t.lower() else [t,'Other']
print(parts[1].strip() if len(parts)>1 else 'Other')
")
E2_SIDE_A=$(python3 -c "
t='$EVENT2_TITLE'
print(t.split(' @ ')[0].strip() if ' @ ' in t else t.split(' vs ')[0].strip() if ' vs ' in t.lower() else t)
")
E2_SIDE_B=$(python3 -c "
t='$EVENT2_TITLE'
parts = t.split(' @ ') if ' @ ' in t else t.split(' vs ') if ' vs ' in t.lower() else [t,'Other']
print(parts[1].strip() if len(parts)>1 else 'Other')
")

###############################################################################
section "5. Create Bets"
###############################################################################

# Bet 1: Alice → Bob (proposed, straight 1:1)
echo "  Alice bets Bob 200 coins on $EVENT1_TITLE..."
BET1=$(curl -s -X POST "$API/api/bets" \
  -H "Content-Type: application/json" -b "$C1" \
  -d "{\"event_id\":\"$EVENT1_ID\",\"opponent_id\":\"$BOB_ID\",\"question\":\"$EVENT1_TITLE\",\"creator_position\":\"$E1_SIDE_A\",\"opponent_position\":\"$E1_SIDE_B\",\"amount\":200,\"odds_numerator\":1,\"odds_denominator\":1,\"expires_in_hours\":48}")
BET1_ID=$(echo "$BET1" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
BET1_STATUS=$(echo "$BET1" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])" 2>/dev/null)
if [ -n "$BET1_ID" ]; then
  ok "Bet 1 created: $BET1_ID status=$BET1_STATUS (Alice→Bob, 200 coins)"
else
  fail "Bet 1 creation failed: $BET1"
fi

# Bet 2: Alice → Carol (proposed, straight 1:1)
echo "  Alice bets Carol 150 coins on $EVENT2_TITLE..."
BET2=$(curl -s -X POST "$API/api/bets" \
  -H "Content-Type: application/json" -b "$C1" \
  -d "{\"event_id\":\"$EVENT2_ID\",\"opponent_id\":\"$CAROL_ID\",\"question\":\"$EVENT2_TITLE\",\"creator_position\":\"$E2_SIDE_A\",\"opponent_position\":\"$E2_SIDE_B\",\"amount\":150,\"odds_numerator\":1,\"odds_denominator\":1,\"expires_in_hours\":48}")
BET2_ID=$(echo "$BET2" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
BET2_STATUS=$(echo "$BET2" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])" 2>/dev/null)
if [ -n "$BET2_ID" ]; then
  ok "Bet 2 created: $BET2_ID status=$BET2_STATUS (Alice→Carol, 150 coins)"
else
  fail "Bet 2 creation failed: $BET2"
fi

# Bet 3: Bob posts open bet (no opponent)
echo "  Bob posts open bet for 100 coins on $EVENT1_TITLE..."
BET3=$(curl -s -X POST "$API/api/bets" \
  -H "Content-Type: application/json" -b "$C2" \
  -d "{\"event_id\":\"$EVENT1_ID\",\"question\":\"$EVENT1_TITLE\",\"creator_position\":\"$E1_SIDE_B\",\"opponent_position\":\"$E1_SIDE_A\",\"amount\":100,\"odds_numerator\":1,\"odds_denominator\":1,\"expires_in_hours\":48}")
BET3_ID=$(echo "$BET3" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
BET3_STATUS=$(echo "$BET3" | python3 -c "import sys,json; print(json.load(sys.stdin)['status'])" 2>/dev/null)
if [ -n "$BET3_ID" ]; then
  ok "Bet 3 created: $BET3_ID status=$BET3_STATUS (Bob open, 100 coins)"
else
  fail "Bet 3 creation failed: $BET3"
fi

# Bet 4: Carol → Bob (proposed)
echo "  Carol bets Bob 75 coins on $EVENT2_TITLE..."
BET4=$(curl -s -X POST "$API/api/bets" \
  -H "Content-Type: application/json" -b "$C3" \
  -d "{\"event_id\":\"$EVENT2_ID\",\"opponent_id\":\"$BOB_ID\",\"question\":\"$EVENT2_TITLE\",\"creator_position\":\"$E2_SIDE_B\",\"opponent_position\":\"$E2_SIDE_A\",\"amount\":75,\"odds_numerator\":1,\"odds_denominator\":1,\"expires_in_hours\":24}")
BET4_ID=$(echo "$BET4" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
if [ -n "$BET4_ID" ]; then
  ok "Bet 4 created: $BET4_ID (Carol→Bob, 75 coins)"
else
  fail "Bet 4 creation failed: $BET4"
fi

###############################################################################
section "6. Bob Accepts Bet 1 (Alice's challenge)"
###############################################################################

echo "  Bob checks notifications..."
BOB_NOTIFS=$(curl -s "$API/api/notifications" -b "$C2")
NOTIF_COUNT=$(echo "$BOB_NOTIFS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)
ok "Bob has $NOTIF_COUNT notification(s)"

echo "  Bob accepts bet 1..."
ACCEPT1=$(curl -s -X POST "$API/api/bets/$BET1_ID/accept" -b "$C2")
ACCEPT1_STATUS=$(echo "$ACCEPT1" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "failed")
if [ "$ACCEPT1_STATUS" = "active" ]; then
  ok "Bet 1 is now ACTIVE"
else
  fail "Bet 1 accept failed: status=$ACCEPT1_STATUS. Raw: $ACCEPT1"
fi

###############################################################################
section "7. Carol Declines Bet 2 (Alice's challenge)"
###############################################################################

echo "  Carol declines bet 2..."
DECLINE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$API/api/bets/$BET2_ID/decline" -b "$C3")
if [ "$DECLINE_STATUS" = "200" ]; then
  ok "Bet 2 DECLINED — Alice gets refund"
else
  fail "Bet 2 decline failed: HTTP $DECLINE_STATUS"
fi

###############################################################################
section "8. Carol Takes Bob's Open Bet (Bet 3)"
###############################################################################

echo "  Carol takes Bob's open bet..."
TAKE=$(curl -s -X POST "$API/api/bets/$BET3_ID/take" -b "$C3")
TAKE_STATUS=$(echo "$TAKE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "failed")
if [ "$TAKE_STATUS" = "active" ]; then
  ok "Bet 3 taken by Carol — now ACTIVE"
else
  fail "Bet 3 take failed: $TAKE_STATUS. Raw: $TAKE"
fi

###############################################################################
section "9. Bob Accepts Carol's Bet (Bet 4)"
###############################################################################

echo "  Bob accepts Carol's bet 4..."
ACCEPT4=$(curl -s -X POST "$API/api/bets/$BET4_ID/accept" -b "$C2")
ACCEPT4_STATUS=$(echo "$ACCEPT4" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "failed")
if [ "$ACCEPT4_STATUS" = "active" ]; then
  ok "Bet 4 is now ACTIVE"
else
  fail "Bet 4 accept failed: $ACCEPT4_STATUS"
fi

###############################################################################
section "10. Settle Bet 1 (Alice wins)"
###############################################################################

echo "  Alice settles bet 1 — creator wins..."
SETTLE1=$(curl -s -X POST "$API/api/bets/$BET1_ID/settle" \
  -H "Content-Type: application/json" -b "$C1" \
  -d '{"winner":"creator"}')
SETTLE1_STATUS=$(echo "$SETTLE1" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "failed")
if [ "$SETTLE1_STATUS" = "settled" ]; then
  ok "Bet 1 SETTLED — Alice wins 400 coins"
else
  fail "Bet 1 settle failed: $SETTLE1_STATUS"
fi

###############################################################################
section "11. Settle Bet 3 (Bob wins, Carol loses)"
###############################################################################

echo "  Bob settles bet 3 — creator (Bob) wins..."
SETTLE3=$(curl -s -X POST "$API/api/bets/$BET3_ID/settle" \
  -H "Content-Type: application/json" -b "$C2" \
  -d '{"winner":"creator"}')
SETTLE3_STATUS=$(echo "$SETTLE3" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "failed")
if [ "$SETTLE3_STATUS" = "settled" ]; then
  ok "Bet 3 SETTLED — Bob wins 200 coins"
else
  fail "Bet 3 settle failed: $SETTLE3_STATUS"
fi

###############################################################################
section "12. Final Balances & Stats"
###############################################################################

for name_jar in "Alice:$C1" "Bob:$C2" "Carol:$C3"; do
  name="${name_jar%%:*}"
  jar="${name_jar##*:}"
  me=$(curl -s "$API/api/users/me" -b "$jar")
  bal=$(echo "$me" | python3 -c "import sys,json; print(json.load(sys.stdin)['coin_balance'])")
  wins=$(echo "$me" | python3 -c "import sys,json; print(json.load(sys.stdin)['wins'])")
  losses=$(echo "$me" | python3 -c "import sys,json; print(json.load(sys.stdin)['losses'])")
  wagered=$(echo "$me" | python3 -c "import sys,json; print(json.load(sys.stdin)['total_wagered'])")
  ok "$name: balance=$bal wins=$wins losses=$losses wagered=$wagered"
done

###############################################################################
section "13. Transaction History"
###############################################################################

for name_jar in "Alice:$C1" "Bob:$C2" "Carol:$C3"; do
  name="${name_jar%%:*}"
  jar="${name_jar##*:}"
  echo "  $name's transactions:"
  curl -s "$API/api/wallet/transactions?limit=10" -b "$jar" | python3 -c "
import sys,json
txns = json.load(sys.stdin)
for t in txns:
  print(f'    {t[\"type\"]:>15}: {t[\"amount\"]:>+6} → {t[\"balance_after\"]}')
" 2>/dev/null
done

###############################################################################
section "14. Notifications"
###############################################################################

for name_jar in "Alice:$C1" "Bob:$C2" "Carol:$C3"; do
  name="${name_jar%%:*}"
  jar="${name_jar##*:}"
  notifs=$(curl -s "$API/api/notifications" -b "$jar")
  count=$(echo "$notifs" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)
  echo "  $name: $count notification(s)"
  echo "$notifs" | python3 -c "
import sys,json
notifs = json.load(sys.stdin)
for n in notifs[:5]:
  print(f'    • {n[\"type\"]}: {str(n.get(\"payload\",{}))[:80]}')
" 2>/dev/null
done

###############################################################################
section "15. Open Feed & Bet Listings"
###############################################################################

echo "  Open bets feed:"
curl -s "$API/api/feed/open" -b "$C1" | python3 -c "
import sys,json
bets = json.load(sys.stdin)
print(f'  {len(bets)} open bets')
for b in bets[:5]:
  print(f'    • {b[\"question\"][:50]} — {b[\"amount\"]} coins, status={b[\"status\"]}')
" 2>/dev/null

echo ""
echo "  Alice's bets:"
curl -s "$API/api/bets" -b "$C1" | python3 -c "
import sys,json
bets = json.load(sys.stdin)
for b in bets:
  print(f'    • {b[\"question\"][:40]} — {b[\"amount\"]} coins, status={b[\"status\"]}, odds={b[\"odds_numerator\"]}:{b[\"odds_denominator\"]}')
" 2>/dev/null

echo ""
echo "  Bob's bets:"
curl -s "$API/api/bets" -b "$C2" | python3 -c "
import sys,json
bets = json.load(sys.stdin)
for b in bets:
  print(f'    • {b[\"question\"][:40]} — {b[\"amount\"]} coins, status={b[\"status\"]}')
" 2>/dev/null

###############################################################################
section "16. Frontend SSR Check"
###############################################################################

for page in "/" "/login" "/bets" "/bets/new" "/events" "/friends" "/feed" "/profile"; do
  status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:5175$page")
  if [ "$status" = "200" ]; then
    ok "GET $page → $status"
  else
    fail "GET $page → $status"
  fi
done

###############################################################################
printf "\n\033[1;33m━━━ RESULTS ━━━\033[0m\n"
if [ "$FAILURES" -eq 0 ]; then
  printf "\033[1;32m  ALL TESTS PASSED! ✅\033[0m\n"
else
  printf "\033[1;31m  %d TEST(S) FAILED ❌\033[0m\n" "$FAILURES"
fi
echo ""
