(() => {
    const canvas = document.getElementById("main_canvas");
    const ctx = canvas.getContext("2d");

    const nRows = 4;
    const nCols = 5;
    const widthBorder = 4;
    const heightBorder = 4;
    const width = canvas.width - 2 * widthBorder;
    const height = canvas.height - 2 * heightBorder;
    const widthCol = width / nCols;
    const heightRow = height / nRows;

    ctx.fillStyle = "orange";
    ctx.fillRect(0,0, canvas.width, canvas.height);

    ctx.save();
    ctx.translate(widthBorder, heightBorder);
    for (let row = 0; row < nRows; row++){
        for (let col = 0; col < nCols; col++){
            const x = widthCol * col + widthBorder;
            const y = heightRow * row + heightBorder;
            ctx.clearRect(x, y, widthCol - 2 * widthBorder, heightRow - 2 * heightBorder);
        }
    }
    ctx.restore();
})()
