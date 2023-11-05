// Create a connection to a websocket and
// try to reconnect when the websocket is closed.
let socket;
function connect_ws(loc, messageCallback){
    let reconnectionTimerId;

    const requestUrl = `${loc.origin.replace("http", "ws")}/_websocket`

    function log(message) {
        console.info("[ws_connection] ", message);
    }

    function connect() {
        if (socket) {
            socket.close();
        }

        socket = new WebSocket(requestUrl);

        socket.onopen = function() {
            log("connection open");
        }

        socket.onclose = function() {
            log("connection lost, reconnecting...");

            clearTimeout(reconnectionTimerId);

            reconnectionTimerId = setTimeout(() => {
                connect();
            }, 1000);
        }

        socket.onmessage = function (event) {
            messageCallback(event.data);
        };
    }

    connect();
}

function send_message(message){
    if (!socket){
        console.error("[ws_connection] socket does not exist");
        return;
    }

    socket.send(message);
}

export {connect_ws, send_message};
