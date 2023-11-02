function connect_ws(loc, messageCallback){
    let socket, reconnectionTimerId;

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

export {connect_ws};
