import { Repairer } from "image-repair";

export class DrawingCanvas {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    holeRect?: {x: number, y: number, w: number, h: number};

    constructor(canvasId: string) {
        this.canvas = document.getElementById(canvasId) as HTMLCanvasElement;
        this.ctx = this.canvas.getContext("2d");
    }

    width() {
        return this.canvas.width;
    }

    height() {
        return this.canvas.height;
    }

    center() {
        return {x: this.width() / 2, y: this.height() / 2};
    }

    drawBackground() {
        this.ctx.fillStyle = "#000000";
        this.ctx.fillRect(0, 0, this.width(), this.height());
    }

    loadImage(src: string) {
        let img = new Image();
        return new Promise<void>( (resolve, reject) => {
            img.onload = () => {
                this.ctx.drawImage(img, 0, 0);
                resolve();
            };
            img.onerror = reject;
            img.src = src;
        });
    }

    drawForeground() {
        this.ctx.fillStyle = "#FF0000";
        const radii = {x: 50, y: 120};
        const rotation = 0;
        const angles = {from: 0, to: 2*Math.PI};
        const center = this.center();
        this.ctx.ellipse(
            center.x, center.y,
            radii.x, radii.y,
            rotation,
            angles.from, angles.to);
        this.ctx.fill();
    }

    process() {
        if (typeof this.holeRect === typeof undefined) {
            throw new Error("There is no hole defined for this canvas!");
        }
        const repairer = new Repairer(this.ctx.canvas.id, this.holeRect.x, this.holeRect.y, this.holeRect.w, this.holeRect.h);
        repairer.repair();
        repairer.free();
    }
}