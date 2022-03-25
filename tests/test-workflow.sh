#!env bash

HOST="http://localhost:8080/api/v0"
USER="test@example.com"
PASSWORD="test"

# -----------------------------------HELPERS-----------------------------------
RES=
TOKEN=
BOLDGREEN="\033[0;1;32m"
BOLDRED="\033[0;1;31m"
NORMAL="\033[0;39m"
function title {
	echo
	echo -e "${BOLDGREEN}${@}${NORMAL}"
}
function error {
	echo -e "${BOLDRED}${@}${NORMAL}" >&2
	echo "$RES" | jq . >&2
	exit 1
}
function try {
	RES=$("$@")
}
function from_json_resp {
	echo "$RES" | jq -r "$@"
}
function assert {
	[[ ! "x$1" = "x" ]] && [[ ! "x$2" = "x" ]] && [[ "x$1" = "x$2" ]]
}
function press_key_to_cont {
	echo Press a key to continue...
	read -sn 1
}
function xh_authed {
	xh --auth-type bearer --auth "$TOKEN" "$@"
}


# -----------------------------------------------------------------------------
title Logging in
echo "   USER: $USER"
try xh POST $HOST/users/auth/login email="$USER" password="$PASSWORD" || \
	error Failed to login

IS_AUTH=$(from_json_resp .auth)
TOKEN=$(from_json_resp .token)    # set TOKEN here
assert "$IS_AUTH" "true" || \
	error Could not login as $USER


# -----------------------------------------------------------------------------
title Getting user details
try xh GET $HOST/users/$USER || \
	error Failed to create feed 

RES_USER=$(from_json_resp .email)
RES_CREATED_AT=$(from_json_resp .created_at)
cat <<__EOF__
         USER: $RES_USER
   CREATED_AT: $RES_CREATED_AT
__EOF__


# -----------------------------------------------------------------------------
CAPTION="My test feed"
IMAGE=~/Pictures/19425_en_1.jpg


# -----------------------------------------------------------------------------
title Creating feed
cat <<__EOF__
      CAPTION: $CAPTION
__EOF__

try xh_authed POST $HOST/feed caption="$CAPTION" || \
	error Failed to create feed
UPLOAD_URL=$(from_json_resp .url)
FEED_ID=$(from_json_resp .id)


# -----------------------------------------------------------------------------
title Uploading feed image
echo "   IMAGE: $IMAGE"
try xh PUT "$UPLOAD_URL" Content-Type:image/jpeg < $IMAGE || \
	error Failed to upload $IMAGE


# -----------------------------------------------------------------------------
title Getting feed details
try xh_authed GET $HOST/feed/$FEED_ID || \
	error Failed to get feed $FEED_ID details
RES_CAPTION=$(from_json_resp .caption)
RES_URL=$(from_json_resp .url)
cat <<__EOF__
        ID: $FEED_ID
   CAPTION: $RES_CAPTION
       URL: $RES_URL
__EOF__

# -----------------------------------------------------------------------------
title Getting thumbnail of feed $FEED_ID
xh --body GET $HOST/feed/$FEED_ID/thumbnail


# -----------------------------------------------------------------------------
title Deleting the feed
press_key_to_cont
try xh_authed DELETE $HOST/feed/$FEED_ID || \
	error Failed to delete feed $FEED_ID
echo Deleted feed $FEED_ID

# -----------------------------------------------------------------------------
title Getting latest feeds without auth
press_key_to_cont
xh --body GET $HOST/feed


# -----------------------------------------------------------------------------
title Getting latest feeds with auth
press_key_to_cont
xh_authed --body GET $HOST/feed


# -----------------------------------------------------------------------------
title Getting latest feed thumbnails
press_key_to_cont
xh --body GET $HOST/feed/thumbnails
