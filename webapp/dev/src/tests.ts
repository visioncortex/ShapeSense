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
    src: string,
}

export const shape1TestInputs: Array<FileTestInput> = [
    {
        canvasId: "top left",
        holeRect: {x: 25, y: 30, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "top center",
        holeRect: {x: 45, y: 30, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "top right",
        holeRect: {x: 75, y: 30, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "middle left",
        holeRect: {x: 20, y: 50, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "random",
        holeRect: {x: 15, y: 55, w: 35, h: 40},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "middle right",
        holeRect: {x: 75, y: 50, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "bottom left",
        holeRect: {x: 20, y: 80, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "bottom center",
        holeRect: {x: 50, y: 80, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
    {
        canvasId: "bottom right",
        holeRect: {x: 70, y: 80, w: 30, h: 30},
        src: "./assets/shape1.png"
    },
];