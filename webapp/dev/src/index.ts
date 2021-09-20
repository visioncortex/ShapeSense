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

testInputs.forEach( (testInput, i) => {
    createHTMLCanvasElement(testInput.canvasId);

    const testCanvas = new DrawingCanvas(testInput.canvasId);

    testCanvas.drawBackground();

    if (typeof testInput.drawForeground === typeof undefined) {
        testCanvas.drawForeground();
    } else {
        testInput.drawForeground(testCanvas);
    }

    testCanvas.holeRect = testInput.holeRect;
    
    try {
        testCanvas.process();
        console.log("%c Test " + testInput.canvasId + " has no errors!", "color: #00FF00");
    } catch (e) {
        console.error(e);
        console.log("%c Test " + testInput.canvasId + " failed!", "color: #FF0000");
    }

    return testCanvas;
});