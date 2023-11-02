import {drawGrid} from "./drawing.js";
import {connect_ws} from "./ws_connection.js";

((loc) => {
    drawGrid();
    const infoarea = document.querySelector("#infoarea");
    connect_ws(loc, (data) => {
        infoarea.value += data + "\r\n";
    });
})(location)
