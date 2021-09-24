import { DrawingCanvas } from "./canvas";
import { testInputs } from "./tests";

const originalCanvas = new DrawingCanvas("original");
originalCanvas.drawBackground();
originalCanvas.drawForeground();

function createHTMLCanvasElement(canvasId: string) {
    let canvas = document.createElement("CANVAS") as HTMLCanvasElement;
    canvas.id = canvasId;
    canvas.width = 200;
    canvas.height = 300;

    let span = document.createElement("SPAN") as HTMLSpanElement;
    span.appendChild(canvas);
    span.title = canvasId; // Set hover text

    document.getElementById("canvasDiv").appendChild(span);
}

let statuses = testInputs.map( (testInput, i) => {
    console.groupCollapsed(testInput.canvasId);

    createHTMLCanvasElement(testInput.canvasId);

    const testCanvas = new DrawingCanvas(testInput.canvasId);

    testCanvas.drawBackground();

    if (typeof testInput.drawForeground !== typeof undefined) {
        testCanvas.drawForeground = () => testInput.drawForeground(testCanvas);
    }
    testCanvas.drawForeground();

    testCanvas.holeRect = testInput.holeRect;

    let status: {canvasId: string, success: boolean};
    
    try {
        testCanvas.process();
        status = {canvasId: testInput.canvasId, success: true};
    } catch (e) {
        console.error(e);
        status = {canvasId: testInput.canvasId, success: false};
    }

    console.groupEnd();
    return status;
});

statuses.forEach((status) => {
    if (!status.success) {
        console.log("%c Test " + status.canvasId + " failed!", "color: #FF0000");
    }
});