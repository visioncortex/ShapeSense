import { DrawingCanvas } from "./canvas";

const originalCanvas = new DrawingCanvas("original");
originalCanvas.drawBackground();
originalCanvas.drawForeground();

interface TestInput {
    canvasId: string;
    holeRect?: {x: number, y: number, w: number, h: number};
}

const testInputs: Array<TestInput> = [
    {
        canvasId: "test-0",
        holeRect: {x: 70, y: 5, w: 60, h: 40},
    },
    {
        canvasId: "test-1",
        holeRect: {x: 45, y: 10, w: 60, h: 40},
    },
];

testInputs.forEach( (testInput, i) => {
    const testCanvas = new DrawingCanvas(testInput.canvasId);

    // Set hover text
    (testCanvas.canvas.parentElement as HTMLSpanElement).title = testInput.canvasId;

    testCanvas.drawBackground();
    testCanvas.drawForeground();
    testCanvas.holeRect = testInput.holeRect;
    
    try {
        testCanvas.process();
        console.log("%c Test " + i + " passed!", "color: #00FF00");
    } catch (e) {
        console.error(e);
        console.log("%c Test " + i + " failed!", "color: #FF0000");
    }

    return testCanvas;
});