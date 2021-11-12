import { DisplaySelector } from "shape-completion";
import { DrawingCanvas } from "./canvas";
import { setUpCustomTest } from "./custom";
import { runPredefinedTests } from "./predefined";
import { TestInput } from "./tests";

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
    switch (document.body.id) {
        case "index":
        case "shape":
            await runPredefinedTests();
    }
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
            switch (document.body.id) {
                case "index":
                case "shape":
                    await runPredefinedTests();
            }
        };
        continue;
    }
}
// End of Controls GUI

export const originalCanvas = new DrawingCanvas("original");

export function process(canvas: DrawingCanvas, testInput: TestInput) {
    canvas.holeRect = testInput.holeRect;

    let status: {testInput: TestInput, success: boolean};
    
    try {
        canvas.process(controls.displaySelector, controls.displayTangents, controls.displayControlPoints);
        status = {testInput, success: true};
    } catch (e) {
        console.error(e);
        status = {testInput, success: false};
    }

    return status;
}

switch (document.body.id) {
    case "index":
    case "shape":
        runPredefinedTests();
        break;

    case "custom":
        setUpCustomTest();
        break;

    default:
        console.error("Unknown document body id.");
}