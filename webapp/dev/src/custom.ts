import { TestInput } from './tests';
import { process, originalCanvas } from ".";
import { DrawingCanvas } from "./canvas";

let fileInput: HTMLInputElement, testCanvas: DrawingCanvas, holeWidthInput: HTMLInputElement, holeHeightInput: HTMLInputElement;
let imageSrc: string;

let processCounter = 0;

const clipValue = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

function getTestInput(): TestInput {
    let { x, y } = testCanvas.lastMousePosition;
    if (x === NaN || y === NaN) {
        console.log("Last mouse position is NaN. Using default values...");
        ({ x, y } = testCanvas.center());
    }

    let w = parseInt(holeWidthInput.value, 10);
    let h = parseInt(holeHeightInput.value, 10);
    if (w === NaN || h === NaN) {
        console.log("Hole width/height is NaN. Using default values...");
        ([w, h] = [15, 15]);
    }

    const [canvasWidth, canvasHeight] = [testCanvas.width(), testCanvas.height()];
    x = Math.round(clipValue(x - w/2, 0, canvasWidth - w));
    y = Math.round(clipValue(y - h/2, 0, canvasHeight - h));

    return {
        canvasId: "testCanvas",
        holeRect: { x, y, w, h },
    };
}

async function loadImageWithImageSrc() {
    await originalCanvas.loadImage(imageSrc);
    await testCanvas.loadImage(imageSrc);

    let [ width, height ] = [ testCanvas.width(), testCanvas.height() ];
    holeWidthInput.value = Math.round(width*0.3).toString(10);
    holeHeightInput.value = Math.round(height*0.3).toString(10);
}

export function setUpCustomTest() {
    fileInput = document.getElementById("fileInput") as HTMLInputElement;
    testCanvas = new DrawingCanvas("testCanvas");
    holeWidthInput = document.getElementById("holeWidthInput") as HTMLInputElement;
    holeHeightInput = document.getElementById("holeHeightInput") as HTMLInputElement;

    // Handle file upload
    fileInput.addEventListener("change", function (event) {
        event.preventDefault();

        const image = this.files.item(0);
        if (image) {
            console.clear();
            imageSrc = URL.createObjectURL(image);
            loadImageWithImageSrc();
        } else {
            console.error("Cannot open image!");
        }
    });

    // Run the test
    {
    const runTest = (_: MouseEvent) => {
        if (!testCanvas.isKeyDown) {
            return;
        }

        testCanvas.loadImage(imageSrc)
            .then(() => {
                const testInput = getTestInput();
                return process(testCanvas, testInput);
            })
            .then(({testInput, success}) => {
                if (!success) {
                    console.error(`Test #${processCounter} failed.\nTest input is as follows:`);
                    console.dir(testInput);
                }
                processCounter += 1;
            })
            .catch(console.error);
    };
    testCanvas.canvas.addEventListener("mousemove", runTest);
    testCanvas.canvas.addEventListener("mousedown", runTest);
    }

    // Impose constraint on inputs
    {
    const onchange = (inputElement: HTMLInputElement, inputMax: number) => (_: Event) => {
        let value = parseInt(inputElement.value, 10);
        value = clipValue(value, 1, inputMax);
        inputElement.value = value.toString(10);
    };
    holeWidthInput.onchange = onchange(holeWidthInput, testCanvas.width());
    holeHeightInput.onchange = onchange(holeHeightInput, testCanvas.height());
    }

    // Set up initial case
    imageSrc = "./assets/shape6.png";
    loadImageWithImageSrc().then(() => {
        process(testCanvas, getTestInput());
        console.clear();
    });
}