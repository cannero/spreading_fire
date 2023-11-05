import {drawGrid} from "./drawing.js";
import {connect_ws, send_message} from "./ws_connection.js";

((loc) => {
    drawGrid();

    const infoarea = document.querySelector("#infoarea");
    connect_ws(loc, (data) => {
        infoarea.value += data + "\r\n";
    });

    const start_btn = document.querySelector("#startcalculation");
    start_btn.addEventListener("click", function(e) {
        send_message("[Run calculation]");
    });

})(location)
