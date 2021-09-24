import { DisplaySelector } from "image-repair";
import { DrawingCanvas } from "./canvas";
import { shapeTestInputs, FileTestInput, IndexTestInput, indexTestInputs, TestInput } from "./tests";

// Controls
const displaySelector = DisplaySelector.None;
const displayTangents = false;
// End of Controls

const originalCanvas = new DrawingCanvas("original");

function createHTMLCanvasElement(canvasId: string, index: number) {
    let canvas = document.createElement("CANVAS") as HTMLCanvasElement;
    canvas.id = canvasId;
    canvas.width = originalCanvas.width();
    canvas.height = originalCanvas.height();

    let span = document.createElement("SPAN") as HTMLSpanElement;
    span.appendChild(canvas);
    span.title = canvasId; // Set hover text

    document.getElementById("canvasDiv").appendChild(span);

    if (index % 3 === 2) {
        document.getElementById("canvasDiv").appendChild(
            document.createElement("BR")
        );
    }
}

function process(canvas: DrawingCanvas, testInput: TestInput) {
    canvas.holeRect = testInput.holeRect;

    let status: {canvasId: string, success: boolean};
    
    try {
        canvas.process(displaySelector, displayTangents);
        status = {canvasId: testInput.canvasId, success: true};
    } catch (e) {
        console.error(e);
        status = {canvasId: testInput.canvasId, success: false};
    }

    return status;
}

async function run() {

originalCanvas.drawBackground();

let testInputs: Array<TestInput>;

switch (document.body.id) {
    case "index":
        testInputs = indexTestInputs;
        originalCanvas.drawForeground();
        break;
    default:
        testInputs = shapeTestInputs.get(document.body.id);
        await originalCanvas.loadImage(`./assets/${document.body.id}.png`);
}

Promise.all(testInputs.map( async (testInput, i) => {
    console.groupCollapsed(testInput.canvasId);

    createHTMLCanvasElement(testInput.canvasId, i);

    const testCanvas = new DrawingCanvas(testInput.canvasId);

    testCanvas.drawBackground();

    if (typeof (testInput as IndexTestInput)?.drawForeground !== typeof undefined) {
        testCanvas.drawForeground = () => (testInput as IndexTestInput).drawForeground(testCanvas);
    }

    let src = (testInput as FileTestInput)?.src;
    if (src) {
        try {
            await testCanvas.loadImage(src);
        } catch(e) {
            console.groupEnd();
            return {canvasId: testInput.canvasId, success: false};
        }
    } else {
        testCanvas.drawForeground();
    }

    let status = process(testCanvas, testInput);
    console.groupEnd();
    return status;
}))
.then((statuses) => {
    statuses.forEach((status) => {
        if (!status.success) {
            console.log("%c Test " + status.canvasId + " failed!", "color: #FF0000");
        }
    });
});

} // End of run()

run();