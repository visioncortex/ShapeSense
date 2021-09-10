import { Repairer } from "image-repair";

const inputCanvas = document.getElementById("input") as HTMLCanvasElement;
const outputCtx = inputCanvas.getContext("2d");

const inputCanvasCenter = {x: inputCanvas.width/2, y: inputCanvas.height/2};

function setUpCanvas() {
    outputCtx.fillStyle = "#000000";
    outputCtx.fillRect(0, 0, inputCanvas.width, inputCanvas.height);
}

function drawForeground() {
    outputCtx.fillStyle = "#FF0000";
    const radii = {x: 50, y: 120};
    const rotation = 0;
    const angles = {from: 0, to: 2*Math.PI};
    outputCtx.ellipse(
        inputCanvasCenter.x, inputCanvasCenter.y,
        radii.x, radii.y,
        rotation,
        angles.from, angles.to);
    outputCtx.fill();
}

function process() {
    const maskRect = {x: 370, y: 80, w: 60, h: 40};
    const repairer = new Repairer("input", maskRect.x, maskRect.y, maskRect.w, maskRect.h);

    repairer.repair();
}

setUpCanvas();
drawForeground();
process();