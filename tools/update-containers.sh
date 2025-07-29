#!/bin/bash

LOG_FILE="/var/log/update_containers.log"  

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

cd ..

touch "$LOG_FILE" || {
    echo "ERROR: cannot create log file: $LOG_FILE" >&2
    exit 1
}

PREV_HASH=$(git rev-parse HEAD)
log "Hash before pull: $PREV_HASH"

# Intentar hacer pull del repositorio
log "Doing git pull"
git pull origin main > /dev/null 2>&1
if [ $? -eq 0 ]; then
    log "Git pull exitoso"
else
    log "ERROR during git pull"
    exit 1
fi

CURRENT_HASH=$(git rev-parse HEAD)
log "Hash after pull: $CURRENT_HASH"


if [ "$PREV_HASH" != "$CURRENT_HASH" ]; then
    log "Changes detected, update requiered"
    
    log "Building new image"
    make build-image >> "$LOG_FILE" 2>&1
    if [ $? -eq 0 ]; then
        log "Image built succesfully"
    else
        log "ERROR building image"
        exit 1
    fi

    log "Stoping containers"
    make stop-pod >> "$LOG_FILE" 2>&1
    if [ $? -eq 0 ]; then
        log "Containers stopped succesfully"
    else
        log "ERROR stopping containers"
        exit 1
    fi
 
    log "Starting containers"
    make run-pod >> "$LOG_FILE" 2>&1
    if [ $? -eq 0 ]; then
        log "Containers started succesfully"
    else
        log "ERROR starting containers"
        exit 1
    fi

else
    log "No changes detected, no update required"
fi
