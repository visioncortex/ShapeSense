# Shape Completion by Curve Stitching

The ShapeCompletion project aims at recovering (completing) the **structure** of a shape after a portion of it is erased. In other words, given a shape with arbitrarily "discontinued parts", assuming that we have **no prior knowledge** about the shape, how can we **reasonably connect the endpoints** at the discontinuities?

![Hole showcase](images/hole_showcase.png)
A shape with a hole. There are 6 endpoints in this case.

Demo images will be used throughout this documentation. Black pixels denote the background (which is completely ignored). Red pixels denote the rasterized *shape*. White pixels denote the *hole*. The (imaginary) intersection of the shape and the hole is the aforementioned "discontinued parts", which is what we try to recover.

![Recovered shape showcase](images/recovered_shape.png)
An example of completed shape.

Blue pixels will be used to denote the outline of the recovered parts of the shape.

The whole process of shape completion involves intrapolating the missing outline and then filling the pixels in the hole with appropriate colors.

# Simple Shape Completion

Let's begin the experiment with simple shapes, like an ellipse.

![Ellipse ground truth](images/simple/ellipse_groundtruth.png)
![Ellipse with hole](images/simple/ellipse_with_hole.png)
The first image is the ground truth (for reference only). The second image is the input to the ShapeCompletion pipeline.

## Path Preprocessing

The first stage of the pipeline is to obtain and process the paths (curves) representing the existing outline of the shape.

[Vision Cortex's library](https://github.com/visioncortex/visioncortex) provides the necessary utilities to extract raw paths from an image.

![Path segments after preprocessing](images/simple/ellipse_preprocessed_process.png)
Yellow pixels denote the identified outline of the shape after simplification.

## Tail Tangent Approximation

The first step in the pipeline is to extract the two curves from the two endpoints; smoothing is performed to better approximate the tangents near the endpoints (*tails* of the whole curve). After this step, we will obtain two tangents (2-D direction vectors), one at each tail. We will call these tangents *tail tangents*.

![Tail tangent approximation](images/simple/tail_tangent_approx.png)
After discrete rasterization, even a theoretically smooth curve will contain sharp corners. The naive approach is to simply take A as the tail tangent, but better (more practical/useful) approximations may be obtained by taking more subsequent segments into account (e.g. B,C,D).

A number of factors determine the accuracy and robustness of tail tangent approximation. Our implementation takes into account how many points to consider from the tails, how long should the segments being considered accumulate to, and how the weights for each segment should differ towards the tails.

<hr>

## Intrapolation

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

## Color filling

To fill the hole with appropriate colors, we define three element types: *Blank*, *Structure*, and *Texture*.

Element     |Description
:-----------|:----------
Blank       |Background pixels; Default elements in the hole.
Structure   |Outline of the shape; The intrapolated curve(s) obtained above is rasterized and drawn onto the hole.
Texture     |Solid part of the shape; To be filled in this section.

The Structure elements divide the hole into several subregions. Each of these subregions is either Blank or Texture elements.

![Hole of ellipse divided into two subregions](images/simple/filling_subregions.png)

In our example, the hole is divided by the intrapolated curve into two subregions. In order to guess whether to fill the subregions with Blank or Texture elements, we take majority votes among the pixels around the hole.

For the bottom subregion, the pixels right outside the bottom boundary are mostly red (Texture), therefore this subregion is classified as Texture and filled with Texture elements.

For the top subregion, the pixels outside the left, top, and right sides of the boundary are considered. Most of those pixels are background (Blank), so this subregion is classified as Blank.

After filling, the shape of the ellipse is completed, as follows:

![Complete ellipse](images/simple/ellipse_complete.png)

<hr>

If we move the hole around, shape completion yields the following results:

![Complete ellipses with different holes](images/simple/ellipse_diff_holes.png)

# Complex Shape Completion

The process of shape completion shown above has been rather straightforward because there is a strong assumption - the hole cuts the shape at exactly 2 endpoints only. Consider the following case:

![Ellipse with a long hole cutting its boundary at 4 endpoints](images/complex/ellipse_hole_across.png)
At a glance, we can tell how should the endpoints be grouped - top-left with bottom-left, top-right with bottom-right, but how can we model the problem to match the endpoints such that the result of color filling always makes sense?

## Endpoint Matching

### Failed attempt: Local Proximity

An intuitive approach to perform matching may be by endpoint proximity in a greedy manner. If we simply connect each endpoint to its nearest neighbor, the correct matching is found for the above case. However, this approach ceases to work for the following case:

![Tall hole over ellipse.](images/complex/ellipse_local_proximity_counterexample.png)
The correct matching seems to be (top-left with bottom-left, top-right with bottom-right), but the top two endpoints are the closest.

### Avoiding Intersections

Problematic matchings are the ones that lead to intersecting curves. If intersection occurs, the resulting shape deforms and there may be subregions that are surrounded by others, leading to challenges in color filling. Therefore, the key of endpoint matching lies in **avoiding intersections**.

Before intrapolation, some intersecting curves can already be identified by looking at endpoint connections that intersect.

Imagine we have 4 endpoints A, B, C, and D. If the line segment AB intersects with CD, then the curve intrapolated from A to B must intersect with that from C to D.

![Line intersection among endpoints implies intersection of intrapolated curves](images/complex/line_intersect_implies_curve_intersect.png)

Therefore, the first step to avoid intersecting curves is to filter out matchings that contain intersecting lines.

This [webpage](https://prase.cz/kalva/putnam/psoln/psol794.html) shows that minimizing the total length of endpoint connections is equivalent to finding a matching with no intersecting connections. Hence the problem is reduced to a [Euclidean Bipartite Matching Problem](https://core.ac.uk/download/pdf/82212931.pdf), i.e. optimizing the global weights over matchings. The [Hungarian algorithm](https://en.wikipedia.org/wiki/Hungarian_algorithm) is used to solve such a problem.

The rest of the intersecting curves have to be caught and filtered out after intrapolation. Bézier curve intersection can be detected by a recursive method called [De Casteljau's (Bézier Clipping) algorithm](https://en.wikipedia.org/wiki/De_Casteljau%27s_algorithm), which is implemented in [flo_curves](https://crates.io/crates/flo_curves), the Bézier curve library we use.