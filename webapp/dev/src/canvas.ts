import { DisplaySelector, Repairer, RepairerConfig } from "image-repair";

export class DrawingCanvas {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    holeRect?: {x: number, y: number, w: number, h: number};
    lastMousePosition: {x: number, y: number} = {x: NaN, y: NaN};
    onUpdateLastMousePosition: Array<(lastMousePosition: {x: number, y: number}) => void> = [];
    isKeyDown: boolean = false;

    constructor(canvasId: string) {
        this.canvas = document.getElementById(canvasId) as HTMLCanvasElement;
        this.ctx = this.canvas.getContext("2d");

        this.canvas.addEventListener("mousedown", (_) => this.isKeyDown = true);
        this.canvas.addEventListener("mouseup", (_) => this.isKeyDown = false);

        const setLastMousePosition = (event: MouseEvent) => {
            if (!this.isKeyDown) {
                return;
            }

            const rect = this.canvas.getBoundingClientRect();
            this.lastMousePosition = {
                x: (event.clientX - rect.left) / (rect.right - rect.left) * this.canvas.width,
                y: (event.clientY - rect.top) / (rect.bottom - rect.top) * this.canvas.height
            };

            for (let listener of this.onUpdateLastMousePosition) {
                listener(this.lastMousePosition);
            }
        }
        this.canvas.addEventListener("mousemove", setLastMousePosition);
        this.canvas.addEventListener("mousedown", setLastMousePosition);
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
                this.canvas.width = img.width;
                this.canvas.height = img.height;
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

    process(displaySelector: DisplaySelector, displayTangents: boolean, displayControlPoints: boolean) {
        if (typeof this.holeRect === typeof undefined) {
            throw new Error("There is no hole defined for this canvas!");
        }
        const config = new RepairerConfig(this.canvas.id)
            .displaySelector(displaySelector)
            .displayTangents(displayTangents)
            .displayControlPoints(displayControlPoints)
            .holeRect(
                this.holeRect.x,
                this.holeRect.y,
                this.holeRect.w,
                this.holeRect.h,
            );
        Repairer.repair_with_config(config);
    }

    addUpdateLastMousePositionListener(listener: (lastMousePosition: { x: number, y: number }) => void) {
        this.onUpdateLastMousePosition.push(listener);
    }
}