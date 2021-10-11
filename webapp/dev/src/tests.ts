import { DrawingCanvas } from "./canvas";

export interface TestInput {
    canvasId: string;
    holeRect: {x: number, y: number, w: number, h: number};
}

export interface IndexTestInput extends TestInput {
    drawForeground?: (canvas: DrawingCanvas) => void;
}

export const indexTestInputs: Array<IndexTestInput> = [
    {
        canvasId: "top center",
        holeRect: {x: 70, y: 10, w: 60, h: 40},
    },
    {
        canvasId: "top left",
        holeRect: {x: 45, y: 10, w: 60, h: 40},
    },
    {
        canvasId: "top right",
        holeRect: {x: 95, y: 10, w: 60, h: 40},
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 70, y: 250, w: 60, h: 40},
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 45, y: 250, w: 60, h: 40},
    },
    {
        canvasId: "bottom right",
        holeRect: {x: 95, y: 250, w: 60, h: 40},
    },
    {
        canvasId: "middle left",
        holeRect: {x: 25, y: 130, w: 60, h: 40},
    },
    {
        canvasId: "middle right",
        holeRect: {x: 115, y: 130, w: 60, h: 40},
    },
    {
        canvasId: "thin",
        holeRect: {x: 70, y: 10, w: 60, h: 40},
        drawForeground: (canvas) => {
            canvas.ctx.fillStyle = "#FF0000";
            const radii = {x: 10, y: 120};
            const rotation = 0;
            const angles = {from: 0, to: 2*Math.PI};
            const center = canvas.center();
            canvas.ctx.ellipse(
                center.x, center.y,
                radii.x, radii.y,
                rotation,
                angles.from, angles.to);
            canvas.ctx.fill();
        }
    },
    {
        canvasId: "4 endpoints (1)",
        holeRect: {x: 25, y: 50, w: 150, h: 40},
    },
    {
        canvasId: "4 endpoints (2)",
        holeRect: {x: 25, y: 130, w: 150, h: 40},
    },
    {
        canvasId: "4 endpoints (3)",
        holeRect: {x: 25, y: 190, w: 150, h: 40},
    },
    {
        canvasId: "rectangle top left",
        holeRect: {x: 10, y: 10, w: 70, h: 70},
        drawForeground:(canvas) => {
            canvas.ctx.fillStyle = "#FF0000";
            canvas.ctx.fillRect(40, 40, 120, 220);
            canvas.ctx.fill();
        },
    },
    {
        canvasId: "rectangle middle left (1)",
        holeRect: {x: 10, y: 60, w: 70, h: 70},
        drawForeground:(canvas) => {
            canvas.ctx.fillStyle = "#FF0000";
            canvas.ctx.fillRect(40, 40, 120, 220);
            canvas.ctx.fill();
        }
    },
    {
        canvasId: "rectangle middle left (2)",
        holeRect: {x: 10, y: 110, w: 70, h: 70},
        drawForeground:(canvas) => {
            canvas.ctx.fillStyle = "#FF0000";
            canvas.ctx.fillRect(40, 40, 120, 220);
            canvas.ctx.fill();
        }
    },
    {
        canvasId: "rectangle bottom left",
        holeRect: {x: 10, y: 210, w: 70, h: 70},
        drawForeground:(canvas) => {
            canvas.ctx.fillStyle = "#FF0000";
            canvas.ctx.fillRect(40, 40, 120, 220);
            canvas.ctx.fill();
        }
    },
];

export interface FileTestInput extends TestInput {
}

export const shapeTestInputs: Map<string, Array<FileTestInput> > = new Map();

shapeTestInputs.set("shape1", [
    {
        canvasId: "top left",
        holeRect: {x: 25, y: 30, w: 30, h: 30},
    },
    {
        canvasId: "top center",
        holeRect: {x: 45, y: 30, w: 30, h: 30},
    },
    {
        canvasId: "top right",
        holeRect: {x: 75, y: 30, w: 30, h: 30},
    },
    {
        canvasId: "middle left",
        holeRect: {x: 20, y: 50, w: 30, h: 30},
    },
    {
        canvasId: "random",
        holeRect: {x: 15, y: 55, w: 35, h: 40},
    },
    {
        canvasId: "middle right",
        holeRect: {x: 75, y: 50, w: 30, h: 30},
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 20, y: 80, w: 30, h: 30},
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 50, y: 80, w: 30, h: 30},
    },
    {
        canvasId: "bottom right",
        holeRect: {x: 70, y: 80, w: 30, h: 30},
    },
]);

shapeTestInputs.set("shape2", [
    {
        canvasId: "top left",
        holeRect: {x: 15, y: 5, w: 40, h: 20},
    },
    {
        canvasId: "top center",
        holeRect: {x: 40, y: 5, w: 40, h: 20},
    },
    {
        canvasId: "top right",
        holeRect: {x: 60, y: 10, w: 40, h: 20},
    },
    {
        canvasId: "middle left",
        holeRect: {x: 5, y: 30, w: 25, h: 20},
    },
    {
        canvasId: "random",
        holeRect: {x: 40, y: 35, w: 40, h: 20},
    },
    {
        canvasId: "middle right",
        holeRect: {x: 80, y: 30, w: 30, h: 20},
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 10, y: 60, w: 40, h: 20},
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 30, y: 55, w: 25, h: 15},
    },
    {
        canvasId: "bottom right",
        holeRect: {x: 65, y: 55, w: 40, h: 20},
    },
]);

shapeTestInputs.set("shape3", [
    {
        canvasId: "top left",
        holeRect: {x: 45, y: 15, w: 40, h: 40},
    },
    {
        canvasId: "top center",
        holeRect: {x: 60, y: 15, w: 40, h: 40},
    },
    {
        canvasId: "top right",
        holeRect: {x: 90, y: 35, w: 40, h: 40},
    },
    {
        canvasId: "middle left",
        holeRect: {x: 15, y: 65, w: 40, h: 40},
    },
    {
        canvasId: "random",
        holeRect: {x: 105, y: 45, w: 40, h: 40},
    },
    {
        canvasId: "middle right",
        holeRect: {x: 105, y: 85, w: 40, h: 40},
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 10, y: 130, w: 40, h: 40},
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 50, y: 140, w: 40, h: 40},
    },
    {
        canvasId: "bottom right",
        holeRect: {x: 80, y: 140, w: 40, h: 40},
    },
]);

shapeTestInputs.set("shape4", [
    {
        canvasId: "top left",
        holeRect: {x: 35, y: 35, w: 60, h: 60},
    },
    {
        canvasId: "top center",
        holeRect: {x: 75, y: 25, w: 80, h: 60},
    },
    {
        canvasId: "top right",
        holeRect: {x: 135, y: 35, w: 60, h: 60},
    },
    {
        canvasId: "middle left",
        holeRect: {x: 10, y: 90, w: 60, h: 60},
    },
    {
        canvasId: "6 points",
        holeRect: {x: 45, y: 80, w: 170, h: 40},
    },
    {
        canvasId: "middle right",
        holeRect: {x: 160, y: 90, w: 60, h: 60},
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 20, y: 150, w: 60, h: 60},
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 85, y: 175, w: 60, h: 60},
    },
    {
        canvasId: "bottom right (corner adversarial)",
        holeRect: {x: 160, y: 150, w: 60, h: 60},
    },
    {
        canvasId: "6 points right upper (partition adversarial)",
        holeRect: { x: 45, y: 65, w: 170, h: 40 },
    },
    {
        canvasId: "6 points right ~center",
        holeRect: { x: 45, y: 110, w: 170, h: 40 },
    },
    {
        canvasId: "6 points right lower",
        holeRect: { x: 55, y: 170, w: 170, h: 35 },
    },
    {
        canvasId: "long hole 1",
        holeRect: { x: 15, y: 35, w: 200, h: 90 },
    },
    {
        canvasId: "long hole 2 (adversarial?)",
        holeRect: { x: 15, y: 40, w: 200, h: 40 },
    },
    {
        canvasId: "long hole 3",
        holeRect: { x: 15, y: 50, w: 200, h: 40 },
    },
    {
        canvasId: "long hole 4",
        holeRect: { x: 15, y: 70, w: 200, h: 40 },
    },
    {
        canvasId: "long hole 5",
        holeRect: { x: 15, y: 90, w: 200, h: 40 },
    },
    {
        canvasId: "long hole 6",
        holeRect: { x: 15, y: 120, w: 200, h: 40 },
    },
    {
        canvasId: "long hole 7",
        holeRect: { x: 15, y: 155, w: 200, h: 40 },
    },
]);