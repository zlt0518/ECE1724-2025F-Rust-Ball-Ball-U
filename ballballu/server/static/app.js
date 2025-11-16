// WebSocket connection state
let ws = null;
let sequenceNumber = 0;
let myPlayerId = null;
let lastGameState = null;

// UI Elements cache
const elements = {
    status: document.getElementById('status'),
    connectBtn: document.getElementById('connectBtn'),
    disconnectBtn: document.getElementById('disconnectBtn'),
    joinBtn: document.getElementById('joinBtn'),
    quitBtn: document.getElementById('quitBtn'),
    moveButtons: ['moveUp', 'moveDown', 'moveLeft', 'moveRight', 'stopBtn'].map(id => document.getElementById(id)),
    messageLog: document.getElementById('messageLog'),
    tickValue: document.getElementById('tickValue'),
    playersCount: document.getElementById('playersCount'),
    yourScore: document.getElementById('yourScore'),
    dotsCount: document.getElementById('dotsCount'),
    playersList: document.getElementById('playersList'),
    dotsInfo: document.getElementById('dotsInfo')
};

/**
 * Update connection status indicator
 */
function updateStatus(status, text) {
    elements.status.className = `status ${status}`;
    elements.status.textContent = `● ${text}`;
}

/**
 * Log a message to the message log
 */
function logMessage(type, content) {
    const timestamp = new Date().toLocaleTimeString();
    const message = document.createElement('div');
    message.className = `message ${type}`;
    message.innerHTML = `<div class="timestamp">${timestamp}</div><div>${content}</div>`;
    
    // Clear "No messages" placeholder if it exists
    if (elements.messageLog.children.length === 1 && 
        elements.messageLog.children[0].textContent.includes('No messages')) {
        elements.messageLog.innerHTML = '';
    }
    
    elements.messageLog.appendChild(message);
    elements.messageLog.scrollTop = elements.messageLog.scrollHeight;
}

/**
 * Clear all messages from the log
 */
function clearMessages() {
    elements.messageLog.innerHTML = '<div class="empty-state">No messages yet</div>';
}

/**
 * Get WebSocket URL based on current page location
 * If accessed via localhost (SSH tunnel), use ws://localhost:8000
 * Otherwise use the configured server URL
 */
function getWebSocketUrl() {
    const configuredUrl = document.getElementById('serverUrl').value;
    const hostname = window.location.hostname;
    
    // If accessing via localhost (SSH tunnel), use localhost for WebSocket too
    if (hostname === 'localhost' || hostname === '127.0.0.1') {
        // Extract port from configured URL if it's ws://, otherwise default to 8000
        if (configuredUrl.startsWith('ws://')) {
            const url = new URL(configuredUrl);
            return `ws://localhost:${url.port || '8000'}`;
        }
        return 'ws://localhost:8000';
    }
    
    // Otherwise use configured URL as-is
    return configuredUrl;
}

/**
 * Connect to WebSocket server
 */
function connect() {
    const url = getWebSocketUrl();
    updateStatus('connecting', 'Connecting...');
    logMessage('sent', `Connecting to ${url}...`);

    try {
        ws = new WebSocket(url);

        ws.onopen = () => {
            updateStatus('connected', 'Connected');
            logMessage('received', '✅ WebSocket connection established!');
            
            elements.connectBtn.disabled = true;
            elements.disconnectBtn.disabled = false;
            elements.joinBtn.disabled = false;
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                handleServerMessage(data);
            } catch (e) {
                logMessage('error', `Failed to parse message: ${e.message}`);
            }
        };

        ws.onerror = (error) => {
            logMessage('error', `WebSocket error: ${error}`);
        };

        ws.onclose = () => {
            updateStatus('disconnected', 'Disconnected');
            logMessage('received', '🔌 Connection closed');
            
            elements.connectBtn.disabled = false;
            elements.disconnectBtn.disabled = true;
            elements.joinBtn.disabled = true;
            elements.quitBtn.disabled = true;
            elements.moveButtons.forEach(btn => btn.disabled = true);
        };
    } catch (e) {
        updateStatus('disconnected', 'Connection Failed');
        logMessage('error', `Failed to connect: ${e.message}`);
    }
}

/**
 * Disconnect from WebSocket server
 */
function disconnect() {
    if (ws) {
        ws.close();
        ws = null;
    }
}

/**
 * Join the game with player name
 */
function joinGame() {
    const name = document.getElementById('playerName').value || 'TestPlayer';
    const message = { Join: { name: name } };
    sendMessage(message);
    
    elements.quitBtn.disabled = false;
    elements.moveButtons.forEach(btn => btn.disabled = false);
}

/**
 * Send movement input to server
 */
function move(dx, dy) {
    sequenceNumber++;
    const message = {
        Input: {
            input: {
                dx: dx,
                dy: dy,
                sequence_number: sequenceNumber
            }
        }
    };
    sendMessage(message);
}

/**
 * Stop player movement
 */
function stopMovement() {
    sequenceNumber++;
    const message = {
        Input: {
            input: {
                dx: 0,
                dy: 0,
                sequence_number: sequenceNumber
            }
        }
    };
    sendMessage(message);
}

/**
 * Send quit message to server
 */
function sendQuit() {
    const message = { Quit: null };
    sendMessage(message);
    
    elements.quitBtn.disabled = true;
    elements.moveButtons.forEach(btn => btn.disabled = true);
}

/**
 * Send a message to the server
 */
function sendMessage(message) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        const json = JSON.stringify(message);
        ws.send(json);
        logMessage('sent', `📤 ${json}`);
    } else {
        logMessage('error', 'WebSocket is not connected!');
    }
}

/**
 * Handle incoming server messages
 */
function handleServerMessage(data) {
    logMessage('received', `📥 ${JSON.stringify(data, null, 2)}`);

    if (data.StateUpdate) {
        updateGameState(data.StateUpdate.snapshot);
    } else if (data.Welcome) {
        myPlayerId = data.Welcome.player_id;
        logMessage('received', `🎉 Welcome! Your Player ID: ${myPlayerId}`);
    }
}

/**
 * Update game state display with snapshot data
 */
function updateGameState(snapshot) {
    lastGameState = snapshot;

    // Update tick
    elements.tickValue.textContent = snapshot.tick;

    // Update players count
    elements.playersCount.textContent = snapshot.players.length;

    // Update dots count
    elements.dotsCount.textContent = snapshot.dots.length;

    // Update player list
    elements.playersList.innerHTML = '';
    if (snapshot.players.length === 0) {
        elements.playersList.innerHTML = '<li class="empty-state">No players</li>';
    } else {
        snapshot.players.forEach(player => {
            const li = document.createElement('li');
            li.className = 'player-item';
            if (player.id === myPlayerId) {
                li.classList.add('current');
                elements.yourScore.textContent = player.score;
            }
            
            li.innerHTML = `
                <div>
                    <strong>${player.name}</strong> ${player.id === myPlayerId ? '(You)' : ''}
                    <br>
                    <small>ID: ${player.id} | Score: ${player.score} | Radius: ${player.radius.toFixed(1)}</small>
                </div>
                <div>
                    <small>Pos: (${player.x.toFixed(0)}, ${player.y.toFixed(0)})</small>
                </div>
            `;
            elements.playersList.appendChild(li);
        });
    }

    // Update dots info
    if (snapshot.dots.length > 0) {
        elements.dotsInfo.textContent = `${snapshot.dots.length} dots on the map`;
    } else {
        elements.dotsInfo.textContent = 'No dots remaining';
    }
}

/**
 * Initialize keyboard controls
 */
function initKeyboardControls() {
    document.addEventListener('keydown', (e) => {
        if (elements.moveButtons[0].disabled) return; // Movement not enabled
        
        switch(e.key.toLowerCase()) {
            case 'w':
            case 'arrowup':
                e.preventDefault();
                move(0, -1);
                break;
            case 's':
            case 'arrowdown':
                e.preventDefault();
                move(0, 1);
                break;
            case 'a':
            case 'arrowleft':
                e.preventDefault();
                move(-1, 0);
                break;
            case 'd':
            case 'arrowright':
                e.preventDefault();
                move(1, 0);
                break;
            case ' ':
                e.preventDefault();
                stopMovement();
                break;
        }
    });
}

// Update WebSocket URL display on page load
function updateWebSocketUrlDisplay() {
    const url = getWebSocketUrl();
    const wsUrlElement = document.getElementById('wsUrl');
    if (wsUrlElement) {
        wsUrlElement.textContent = url;
    }
}

// Initialize keyboard controls and update URL display when page loads
initKeyboardControls();
updateWebSocketUrlDisplay();

// Update URL display when server URL input changes
document.addEventListener('DOMContentLoaded', () => {
    const serverUrlInput = document.getElementById('serverUrl');
    if (serverUrlInput) {
        serverUrlInput.addEventListener('input', updateWebSocketUrlDisplay);
    }
});

