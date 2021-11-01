import { TestInput } from './tests';
import { process, originalCanvas } from ".";
import { DrawingCanvas } from "./canvas";

let fileInput: HTMLInputElement, testCanvas: DrawingCanvas;
let holeXInput: HTMLInputElement, holeYInput: HTMLInputElement, holeWidthInput: HTMLInputElement, holeHeightInput: HTMLInputElement;
let imageSrc: string;

let processCounter = 0;

const initialHoleRect = { x: 58, y: 14, w: 40, h: 31 };
const clearFirstOutput = false;

const clipValue = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

function getTestInput(): TestInput {
    let x = parseInt(holeXInput.value, 10);
    let y = parseInt(holeYInput.value, 10);

    let w = parseInt(holeWidthInput.value, 10);
    let h = parseInt(holeHeightInput.value, 10);

    holeXInput.value = x.toString(10);
    holeYInput.value = y.toString(10);

    return {
        canvasId: "testCanvas",
        holeRect: { x, y, w, h },
    };
}

async function loadImageWithImageSrc(setDimension: boolean = true) {
    await originalCanvas.loadImage(imageSrc);
    await testCanvas.loadImage(imageSrc);

    let [ width, height ] = [ testCanvas.width(), testCanvas.height() ];
    if (setDimension) {
        holeWidthInput.value = Math.round(width * 0.3).toString(10);
        holeHeightInput.value = Math.round(height * 0.3).toString(10);
    }
}

function runTest() {
    testCanvas.loadImage(imageSrc)
        .then(() => {
            const testInput = getTestInput();
            return process(testCanvas, testInput);
        })
        .then(({ testInput, success }) => {
            if (!success) {
                console.error(`Test #${processCounter} failed.\nTest input is as follows:`);
                console.dir(testInput);
            }
            processCounter += 1;
        })
        .catch(console.error);
}

export function setUpCustomTest() {
    fileInput = document.getElementById("fileInput") as HTMLInputElement;
    testCanvas = new DrawingCanvas("testCanvas");
    holeWidthInput = document.getElementById("holeWidthInput") as HTMLInputElement;
    holeHeightInput = document.getElementById("holeHeightInput") as HTMLInputElement;
    holeXInput = document.getElementById("holeXInput") as HTMLInputElement;
    holeYInput = document.getElementById("holeYInput") as HTMLInputElement;

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

    // Add listener to update input (x,y)
    testCanvas.addUpdateLastMousePositionListener(({x, y}) => {
        let w = parseInt(holeWidthInput.value, 10);
        let h = parseInt(holeHeightInput.value, 10);

        x = x - w/2;
        y = y - h/2;

        const [canvasWidth, canvasHeight] = [testCanvas.width(), testCanvas.height()];
        x = Math.round(clipValue(x, 0, canvasWidth - w));
        y = Math.round(clipValue(y, 0, canvasHeight - h));

        holeXInput.value = x.toString(10);
        holeYInput.value = y.toString(10);
    });

    // Run the test on button press
    document.getElementById("runTestButton").onclick = (_) => runTest();

    // Run the test on mouse events
    {
    const onmouse = (_: MouseEvent) => {
        if (!testCanvas.isKeyDown) {
            return;
        }

        runTest();
    };
    testCanvas.canvas.addEventListener("mousemove", onmouse);
    testCanvas.canvas.addEventListener("mousedown", onmouse);
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

    // Set up export button
    {
    const onclick = (_: MouseEvent) => {
        const link = document.createElement("a");
        link.download = `x${holeXInput.value}_y${holeYInput.value}_w${holeWidthInput.value}_h${holeHeightInput.value}.png`;
        testCanvas.canvas.toBlob((blob) => {
            link.href = URL.createObjectURL(blob);
            link.click();
        }, "image/png");
    };
    let exportButton = document.getElementById("exportButton") as HTMLAnchorElement;
    exportButton.onclick = onclick;
    }

    // Set up initial case
    imageSrc = "./assets/shape6.png";
    holeXInput.value = initialHoleRect.x.toString(10);
    holeYInput.value = initialHoleRect.y.toString(10);
    holeWidthInput.value = initialHoleRect.w.toString(10);
    holeHeightInput.value = initialHoleRect.h.toString(10);

    loadImageWithImageSrc(false).then(() => {
        process(testCanvas, getTestInput());
        if (clearFirstOutput) {
            console.clear();
        }
    });
}