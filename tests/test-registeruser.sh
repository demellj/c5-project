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
title Registering user
echo "       USER: $USER"
echo "   PASSWORD: $PASSWORD"
try xh POST $HOST/users/auth email="$USER" password="$PASSWORD" || \
	error Failed to register user

TOKEN=$(from_json_resp .token)    # set TOKEN here

echo "   TOKEN: $TOKEN"
