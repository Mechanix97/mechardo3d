#!/bin/bash

REPO_DIR="/home/lucas/MECHARDO/mechardo3d"
LOG_FILE="$REPO_DIR/log/update_containers.log"

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

PREV_HASH=$(git rev-parse HEAD 2>/dev/null) || {
    log "ERROR: cannot get initial commit hash"
    exit 1
}
log "Hash before pull: $PREV_HASH"

log "Doing git pull"
git pull origin master >> "$LOG_FILE" 2>&1
[ $? -eq 0 ] || {
    log "ERROR during git pull"
    exit 1
}
log "Git pull successful"

CURRENT_HASH=$(git rev-parse HEAD 2>/dev/null) || {
    log "ERROR: cannot get final commit hash"
    exit 1
}
log "Hash after pull: $CURRENT_HASH"

if [ "$PREV_HASH" != "$CURRENT_HASH" ]; then
    log "Changes detected, update required"
    
    log "Building new image"
    make build-image-prod >> "$LOG_FILE" 2>&1
    [ $? -eq 0 ] || {
        log "ERROR building image"
        exit 1
    }
    log "Image built successfully"

    log "Stopping containers"
    make stop-prod >> "$LOG_FILE" 2>&1
    [ $? -eq 0 ] || {
        log "ERROR stopping containers"
        exit 1
    }
    log "Containers stopped successfully"
 
    log "Starting containers"
    make run-prod >> "$LOG_FILE" 2>&1
    [ $? -eq 0 ] || {
        log "ERROR starting containers"
        exit 1
    }
    log "Containers started successfully"
    exit 0
else
    log "No changes detected, checking container status"
fi

if ! docker compose ps -q mechardo3d > /dev/null 2>&1; then
    log "Container mechardo3d is not running, starting it"
    make run-prod >> "$LOG_FILE" 2>&1
    [ $? -eq 0 ] || {
        log "ERROR starting container"
        exit 1
    }
    log "Container started successfully"
else
    log "Container mechardo3d is already running, no action needed"
fi


log "Script execution finished"
