#!/bin/bash

REPO_DIR="/home/lucas/MECHARDO/mechardo3d"
LOG_FILE="$REPO_DIR/log/update_containers.log"
# Last commit that reached prod. Compared against HEAD after the pull, so a
# deploy that failed is retried on the next run instead of being forgotten.
DEPLOYED_HASH_FILE="$REPO_DIR/log/deployed_hash"

export PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:$PATH

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE" 2>/dev/null
}

touch "$LOG_FILE" || {
    echo "ERROR: cannot create log file: $LOG_FILE" >&2
    exit 1
}

log "Starting script execution"

[ ! -d "$REPO_DIR" ] && {
    log "ERROR: directory $REPO_DIR does not exist"
    exit 1
}

cd "$REPO_DIR" || {
    log "ERROR: cannot access directory $REPO_DIR"
    exit 1
}

log "Doing git pull"
git pull origin master >> "$LOG_FILE" 2>&1
[ $? -eq 0 ] || {
    log "ERROR during git pull"
    exit 1
}
log "Git pull successful"

CURRENT_HASH=$(git rev-parse HEAD 2>/dev/null) || {
    log "ERROR: cannot get commit hash"
    exit 1
}
DEPLOYED_HASH=$(cat "$DEPLOYED_HASH_FILE" 2>/dev/null)
log "HEAD: $CURRENT_HASH, deployed: ${DEPLOYED_HASH:-none}"

if [ "$CURRENT_HASH" = "$DEPLOYED_HASH" ] && docker compose ps -q mechardo3d | grep -q .; then
    log "Already deployed and running, no action needed"
    exit 0
fi

# Builds the image and recreates the container only if the image changed;
# waits for the healthcheck, so a container that never gets healthy counts as
# a failed deploy.
log "Deploying $CURRENT_HASH"
make deploy-prod >> "$LOG_FILE" 2>&1
[ $? -eq 0 ] || {
    log "ERROR deploying, will retry on next run"
    exit 1
}

echo "$CURRENT_HASH" > "$DEPLOYED_HASH_FILE" || {
    log "ERROR: cannot write $DEPLOYED_HASH_FILE"
    exit 1
}
log "Deployed $CURRENT_HASH successfully"
