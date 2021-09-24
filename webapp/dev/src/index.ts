import { DisplaySelector } from "image-repair";
import { DrawingCanvas } from "./canvas";
import { shapeTestInputs, FileTestInput, IndexTestInput, indexTestInputs, TestInput } from "./tests";

// Controls
const displaySelector = DisplaySelector.None;
const displayTangents = false;
// End of Controls

const currentShape = parseInt(document.body.id, 10);
if (currentShape !== NaN) {
    if (currentShape > 1)
        (document.getElementById("back") as HTMLAnchorElement).href = `./shape${currentShape-1}.html`;
    if (currentShape < shapeTestInputs.size)
        (document.getElementById("next") as HTMLAnchorElement).href = `./shape${currentShape+1}.html`;
}

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
        testInputs = shapeTestInputs.get("shape" + document.body.id);
        await originalCanvas.loadImage(`./assets/shape${document.body.id}.png`);
}

const statusPromiseFactories = testInputs.map( (testInput, i) => async () => {
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
});

const dummyPromise = Promise.resolve({canvasId: "", success: true});
statusPromiseFactories.push(() => dummyPromise);

statusPromiseFactories.reduce(
    async (promise, factory) => {
        return promise.then(
            (status) => {
                if (!status.success) {
                    console.log("%c Test " + status.canvasId + " failed!", "color: #FF0000");
                }
                return factory();
            });
        },
    dummyPromise
);

} // End of run()

run();