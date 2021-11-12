# Shape Completion by Curve Stitching

The ShapeCompletion project aims at recovering (completing) the **structure** of a shape after a portion of it is erased. In other words, given a shape with arbitrarily "discontinued parts", assuming that we have **no prior knowledge** about the shape, how can we **reasonably connect the endpoints** at the discontinuities?

![Hole showcase](images/hole_showcase.png)
A shape with a hole. There are 6 endpoints in this case.

Demo images will be used throughout this documentation. Black pixels denote the background (which is completely ignored). Red pixels denote the rasterized *shape*. White pixels denote the *hole*. The (imaginary) intersection of the shape and the hole is the aforementioned "discontinued parts", which is what we try to recover.

![Recovered shape showcase](images/recovered_shape.png)
An example of completed shape.

Blue pixels will be used to denote the outline of the recovered parts of the shape.

The whole process of shape completion involves intrapolating the missing outline and then filling the pixels in the hole with appropriate colors.

## Simple Shape Completion

Let's begin the experiment with simple shapes, like an ellipse.

![Ellipse ground truth](images/simple/ellipse_groundtruth.png)
![Ellipse with hole](images/simple/ellipse_with_hole.png)
The first image is the ground truth (for reference only). The second image is the input to the ShapeCompletion pipeline.

### Path Preprocessing

The first stage of the pipeline is to obtain and process the paths (curves) representing the existing outline of the shape.

[Vision Cortex's library](https://github.com/visioncortex/visioncortex) provides the necessary utilities to extract raw paths from an image.

![Path segments after preprocessing](images/simple/ellipse_preprocessed_process.png)
Yellow pixels denote the identified outline of the shape after simplification.

### Tail Tangent Approximation

The first step in the pipeline is to extract the two curves from the two endpoints; smoothing is performed to better approximate the tangents near the endpoints (*tails* of the whole curve). After this step, we will obtain two tangents (2-D direction vectors), one at each tail. We will call these tangents *tail tangents*.

![Tail tangent approximation](images/simple/tail_tangent_approx.png)
After discrete rasterization, even a theoretically smooth curve will contain sharp corners. The naive approach is to simply take A as the tail tangent, but better (more practical/useful) approximations may be obtained by taking more subsequent segments into account (e.g. B,C,D).

A number of factors determine the accuracy and robustness of tail tangent approximation. Our implementation takes into account how many points to consider from the tails, how long should the segments being considered accumulate to, and how the weights for each segment should differ towards the tails.

<hr>

### Intrapolation

<details>
    <summary>Why "<i>Intra</i>polation"?</summary>
    <p>
        If we considered the existing outline of the shape as separate curves at each endpoint, we would be doing *<b>inter</b>polation* **between** curves. However, in this project, we are focusing on curves that form an outline of a shape, so we argue that we are doing *<b>intra</b>polation* **within** curves.
    </p>
    <hr>
</details>

With the two endpoints and their corresponding tail tangents, we can calculate the missing part in different scenarios. The type of curves used in this project is cubic [Bézier curves](https://en.wikipedia.org/wiki/B%C3%A9zier_curve). To specify such a curve, four points are required.

The first scenario is when the two tail tangents point to the same side with respect to the line connecting the two endpoints (we denote this line as the *base*).

![Intrapolation when both tail tangents point to the same side](images/simple/intrapolate_same_side.png)
Both tail tangents at A and B point to the same side of the red line (the base).

To construct the curve between A and B, we need to identify two *control points* (C<sub>A</sub> and C<sub>B</sub>) between them. In our approach, we started with what we think is intuitive and made tweaks to resolve practical issues; this is what we end up with:

First, find the intersection of the two lines starting at A and B along the corresponding tail tangent. C<sub>A</sub> and C<sub>B</sub> are then set to be the mid-point between A/B and the intersection. If the intersection is too far away (i.e. the two lines are close to parallel), we simply use a point on each of the lines as the control points (e.g. translate A/B along tail tangent by a factor of base length). Either way, if either C<sub>A</sub> or C<sub>B</sub> end up lying outside the hole region, we *retract* it by pushing it towards the endpoint, until it reaches the hole region.

<details>
    <summary>What if the intersection is in the other direction?</summary>
    <img src="./images/simple/intrapolation_bent_outwards.png" alt="Tail tangents bent outwards; pulled back to be perpendicular to the base." />
    <p>
        If the line originating at A and B intersect in the negative direction (as shown above), we simply *correct* them by bending them inwards to be perpendicular with the base.
    </p>
</details>

The other scenario is when the two tail tangents point to different sides of the base, as below.

![Intrapolation with tail tangents pointing to different sides of the base](images/simple/intrapolate_diff_sides.png) A simple dot product operation can be used to detect such a scenario.

In this case, any intersections detected are meaningless. Instead, we divide the curve into two and intrapolate two subcurves from each endpoint to the mid-point of the base as shown above.

The final scenario is trivial to handle: when the lines are coincident, simply connect the endpoints.

![Intrapolation becomes connecting endpoints with a straight line in coincidence](images/simple/intrapolate_coincidence.png)

<hr>

The case of our simple ellipse falls into the first scenario. The intrapolated outline is shown as follows:

![Ellipse after intrapolation](images/simple/ellipse_intrapolated.png)