import { DisplaySelector } from "image-repair";
import { DrawingCanvas } from "./canvas";
import { shapeTestInputs, IndexTestInput, indexTestInputs, TestInput } from "./tests";

interface Indexable {
    [key: string]: any;
}

// Controls
const controls = ({
    displaySelector: DisplaySelector.None,
    displayTangents: false,
    displayControlPoints: false,
}) as Indexable;
// End of Controls

// Controls GUI
const handleCheckboxChange = (checkboxElement: HTMLInputElement) => async (_: Event) => {
    const checked = checkboxElement.checked;
    controls[checkboxElement.id] = checked;
    await run();
};

const controlsDiv = document.getElementById("controls");
for (let controlElem of Array.from(controlsDiv.children)) {
    if (controlElem.tagName === "INPUT") {
        let inputElem = controlElem as HTMLInputElement;
        switch (inputElem.type) {
            case "checkbox":
                inputElem.onchange = handleCheckboxChange(inputElem);
                break;

            default:
        }
        continue;
    }

    if (controlElem.tagName === "SELECT") {
        let selectElem = controlElem as HTMLSelectElement;
        selectElem.onchange = async (_) => {
            switch (selectElem.value) {
                case "None":
                default:
                    controls[selectElem.id] = DisplaySelector.None;
                    break;
                case "Simplified":
                    controls[selectElem.id] = DisplaySelector.Simplified;
                    break;
                case "Smoothed":
                    controls[selectElem.id] = DisplaySelector.Smoothed;
                    break;
            }
            await run();
        };
        continue;
    }
}
// End of Controls GUI

const urlParams = new URLSearchParams(window.location.search);
const shapeId = urlParams.get("id");

const currentShape = parseInt(shapeId, 10);
if (currentShape !== NaN) {
    if (currentShape > 1)
        (document.getElementById("back") as HTMLAnchorElement).href = `./shape.html?id=${currentShape - 1}`;
    if (currentShape < shapeTestInputs.size)
        (document.getElementById("next") as HTMLAnchorElement).href = `./shape.html?id=${currentShape + 1}`;
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

function createShapeLinks() {
    let numShapes = shapeTestInputs.size;
    let divElem = document.getElementById("panel");
    for (let i = 1; i <= numShapes; ++i) {
        let shapeButton = document.createElement("BUTTON");
        shapeButton.innerText = `Shape${i}`;

        let shapeLink = document.createElement("A") as HTMLAnchorElement;
        shapeLink.href = `shape.html?id=${i}`;
        shapeLink.appendChild(shapeButton);
        
        divElem.appendChild(shapeLink);
    }
}
if (document.body.id === "index") createShapeLinks();

function process(canvas: DrawingCanvas, testInput: TestInput) {
    canvas.holeRect = testInput.holeRect;

    let status: {canvasId: string, success: boolean};
    
    try {
        canvas.process(controls.displaySelector, controls.displayTangents, controls.displayControlPoints);
        status = {canvasId: testInput.canvasId, success: true};
    } catch (e) {
        console.error(e);
        status = {canvasId: testInput.canvasId, success: false};
    }

    return status;
}

async function run() {

console.clear();

const canvasDiv = document.getElementById("canvasDiv");
while (canvasDiv.hasChildNodes()) canvasDiv.removeChild(canvasDiv.lastChild);

originalCanvas.drawBackground();

let testInputs: Array<TestInput>;

switch (document.body.id) {
    case "index":
        testInputs = indexTestInputs;
        originalCanvas.drawForeground();
        break;
    default:
        testInputs = shapeTestInputs.get("shape" + currentShape);
        await originalCanvas.loadImage(`./assets/shape${currentShape}.png`);
}

const statusPromiseFactories = testInputs.map( (testInput, i) => async () => {
    console.groupCollapsed(testInput.canvasId);

    createHTMLCanvasElement(testInput.canvasId, i);

    const testCanvas = new DrawingCanvas(testInput.canvasId);

    testCanvas.drawBackground();

    if (typeof (testInput as IndexTestInput)?.drawForeground !== typeof undefined) {
        testCanvas.drawForeground = () => (testInput as IndexTestInput).drawForeground(testCanvas);
    }

    if (document.body.id !== "index") {
        try {
            await testCanvas.loadImage(`./assets/shape${currentShape}.png`);
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